//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use super::genesis_state::ValidatorGenesisState;
use crate::validator::metrics::ValidatorMetrics;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use fastcrypto::Hash;
use gdex_controller::router::ControllerRouter;
use gdex_types::{
    account::ValidatorKeyPair,
    block::{Block, BlockCertificate, BlockDigest, BlockInfo, BlockNumber},
    committee::{Committee, ValidatorName},
    error::GDEXError,
    store::ProcessBlockStore,
    transaction::{ConsensusTransaction, SignedTransaction, Transaction, TransactionDigest},
};
use narwhal_consensus::ConsensusOutput;
use narwhal_executor::{ExecutionIndices, ExecutionState, SerializedTransaction};
use narwhal_types::CertificateDigest;
use std::{
    collections::HashMap,
    path::PathBuf,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{info, trace};
type ExecutionResult = Result<(), GDEXError>;

/// Tracks recently submitted transactions to implement transaction gating
pub struct ValidatorStore {
    /// The transaction map tracks recently submitted transactions
    transaction_cache: Mutex<HashMap<TransactionDigest, Option<BlockDigest>>>,
    block_digest_cache: Mutex<HashMap<BlockDigest, BlockNumber>>,
    // garbage collection depth
    gc_depth: u64,
    pub block_number: AtomicU64,
    pub process_block_store: ProcessBlockStore,
}

impl ValidatorStore {
    pub fn reopen<Path: AsRef<std::path::Path>>(store_path: Path) -> Self {
        let process_block_store: ProcessBlockStore = ProcessBlockStore::reopen(store_path);
        let last_block_info = process_block_store.last_block_info.clone();

        // TODO load the state if last block is not 0, i.e. not at genesis
        let block_number = match last_block_info {
            Ok(o) => {
                if let Some(v) = o {
                    v.block_number
                    // mark for replay somehow here
                } else {
                    0
                }
            }
            Err(_) => 0,
        };

        let block_digest_cache = Mutex::new(HashMap::new());
        // TODO - cleanup writing dummy block
        if block_number == 0 {
            block_digest_cache
                .lock()
                .unwrap()
                .insert(CertificateDigest::new([0; 32]), 0);
        }

        Self {
            transaction_cache: Mutex::new(HashMap::new()),
            block_digest_cache,
            gc_depth: 50,
            process_block_store,
            block_number: AtomicU64::new(block_number),
        }
    }

    pub fn cache_contains_transaction(&self, transaction: &Transaction) -> bool {
        return self
            .transaction_cache
            .lock()
            .unwrap()
            .contains_key(&transaction.digest());
    }

    pub fn cache_contains_block_digest(&self, block_digest: &BlockDigest) -> bool {
        return self.block_digest_cache.lock().unwrap().contains_key(block_digest);
    }

    pub fn insert_unconfirmed_transaction(&self, transaction: &Transaction) {
        self.transaction_cache.lock().unwrap().insert(
            transaction.digest(),
            None, // Insert with no block digest, a digest will be added after confirmation
        );
    }

    pub fn insert_confirmed_transaction(
        &self,
        transaction: &Transaction,
        consensus_output: &ConsensusOutput,
    ) -> Result<(), GDEXError> {
        let block_digest = consensus_output.certificate.digest();
        let transaction_digest = transaction.digest();

        // return an error if transaction has already been seen before
        if let Some(digest) = self.transaction_cache.lock().unwrap().get(&transaction_digest) {
            if digest.is_some() {
                return Err(GDEXError::TransactionDuplicate);
            }
        }

        self.transaction_cache
            .lock()
            .unwrap()
            .insert(transaction_digest, Some(block_digest));
        self.block_digest_cache
            .lock()
            .unwrap()
            .insert(block_digest, consensus_output.consensus_index);
        Ok(())
    }

    pub async fn write_latest_block(
        &self,
        block_certificate: BlockCertificate,
        transactions: Vec<(SerializedTransaction, ExecutionResult)>,
    ) -> (Block, BlockInfo) {
        // TODO - is there a way to acquire a mutable reference to the block-number without demanding &mut self?
        // this would allow us to avoid separate commands to load and add to the counter

        let block_number = self.block_number.load(std::sync::atomic::Ordering::SeqCst);
        let block_digest = block_certificate.digest();

        let block = Block {
            block_certificate: block_certificate.clone(),
            transactions,
        };

        let start = SystemTime::now();
        let validator_system_epoch_time_in_micros = start
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros()
            .try_into()
            .unwrap();

        let block_info = BlockInfo {
            block_number,
            block_digest,
            validator_system_epoch_time_in_micros,
        };

        // write-out the block information to associated stores
        self.process_block_store
            .block_store
            .write(block_number, block.clone())
            .await;
        self.process_block_store
            .block_info_store
            .write(block_number, block_info.clone())
            .await;
        self.process_block_store
            .last_block_info_store
            .write(0, block_info.clone())
            .await;
        // update the block number
        self.block_number.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        (block, block_info)
    }

    pub fn prune(&self) {
        let mut locked_block_digest_cache = self.block_digest_cache.lock().unwrap();
        if locked_block_digest_cache.len() > self.gc_depth as usize {
            let mut threshold = locked_block_digest_cache.values().max().unwrap() - self.gc_depth;
            locked_block_digest_cache.retain(|_k, v| v > &mut threshold);
            self.transaction_cache
                .lock()
                .unwrap()
                .retain(|_k, v| v.is_none() || locked_block_digest_cache.contains_key(&v.unwrap()));
        }
    }
}

impl Default for ValidatorStore {
    fn default() -> Self {
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();
        Self::reopen(store_path)
    }
}

pub type StableSyncValidatorSigner = Pin<Arc<ValidatorKeyPair>>;

/// Encapsulates all state of the necessary state for a validator
/// drives execution of transactions and ensures safety
pub struct ValidatorState {
    // Fixed size, static, identity of the validator
    /// The name of this validator.
    pub name: ValidatorName,
    /// The signature key of the validator.
    pub secret: StableSyncValidatorSigner,
    /// A global lock to halt all transaction/cert processing.
    halted: AtomicBool,
    // Epoch related information.
    /// Committee of this GDEX instance.
    pub committee: ArcSwap<Committee>,
    /// NodeConfig for this node
    /// Controller of various blockchain modules
    pub master_controller: ControllerRouter,
    /// A map of transactions which have been seen
    pub validator_store: ValidatorStore,
    /// Metrics around blockchain operations
    pub metrics: Arc<ValidatorMetrics>,
}

impl ValidatorState {
    // TODO: This function takes both committee and genesis as parameter.
    // Technically genesis already contains committee information. Could consider merging them.
    pub fn new(
        name: ValidatorName,
        secret: StableSyncValidatorSigner,
        genesis: &ValidatorGenesisState,
        store_db_path: &PathBuf,
        metrics: Arc<ValidatorMetrics>,
    ) -> Self {
        ValidatorState {
            name,
            secret,
            halted: AtomicBool::new(false),
            committee: ArcSwap::from(Arc::new(genesis.committee().unwrap())),
            master_controller: genesis.master_controller().clone(),
            validator_store: ValidatorStore::reopen(store_db_path),
            metrics,
        }
    }

    pub fn halt_validator(&self) {
        self.halted.store(true, Ordering::Relaxed);
    }

    pub fn unhalt_validator(&self) {
        self.halted.store(false, Ordering::Relaxed);
    }

    pub fn is_halted(&self) -> bool {
        self.halted.load(Ordering::Relaxed)
    }
}

impl ValidatorState {
    /// Initiate a new transaction.
    pub fn handle_pre_consensus_transaction(&self, signed_transaction: &SignedTransaction) -> Result<(), GDEXError> {
        trace!("Handling a new pre-consensus transaction with the ValidatorState",);
        let transaction = signed_transaction.get_transaction()?;
        self.validator_store.insert_unconfirmed_transaction(transaction);
        Ok(())
    }
}

#[async_trait]
impl ExecutionState for ValidatorState {
    type Transaction = ConsensusTransaction;
    type Error = GDEXError;
    type Outcome = (ConsensusOutput, ExecutionIndices, ExecutionResult);

    async fn handle_consensus_transaction(
        &self,
        consensus_output: &narwhal_consensus::ConsensusOutput,
        execution_indices: ExecutionIndices,
        consensus_transaction: Self::Transaction,
    ) -> Result<Self::Outcome, Self::Error> {
        self.metrics.transactions_executed.inc();

        // deserialize signed transaction
        let signed_transaction = consensus_transaction
            .get_payload()
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let _ = signed_transaction
            .verify_signature()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        // get transaction
        let transaction = signed_transaction.get_transaction()?;

        // cache confirmed transaction
        let uniqueness_check = self
            .validator_store
            .insert_confirmed_transaction(transaction, consensus_output);

        // if transaction is not unique stop execution
        if uniqueness_check.is_err() {
            self.metrics.transactions_executed_failed.inc();
            return Ok((consensus_output.clone(), execution_indices, uniqueness_check));
        }

        let result = self.master_controller.handle_consensus_transaction(transaction);

        if result.is_err() {
            self.metrics.transactions_executed_failed.inc();
        }

        Ok((consensus_output.clone(), execution_indices, result))
    }

    fn ask_consensus_write_lock(&self) -> bool {
        info!("Asking consensus write lock");
        true
    }

    fn release_consensus_write_lock(&self) {
        info!("releasing consensus write lock");
    }

    async fn load_execution_indices(&self) -> Result<ExecutionIndices, Self::Error> {
        Ok(ExecutionIndices::default())
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::Transaction, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod test_validator_state {
    use super::*;
    use crate::{
        builder::genesis_state::GenesisStateBuilder,
        genesis_ceremony::{VALIDATOR_BALANCE, VALIDATOR_FUNDING_AMOUNT},
    };
    use fastcrypto::{generate_production_keypair, DIGEST_LEN};
    use gdex_controller::{
        bank::proto::{create_create_asset_transaction, create_payment_transaction},
        spot::proto::{
            create_cancel_order_transaction, create_create_orderbook_transaction, create_limit_order_transaction,
            create_update_order_transaction,
        },
    };
    use gdex_types::{
        account::ValidatorPubKeyBytes,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        order_book::OrderSide,
        utils,
    };
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::KeyPair;
    use narwhal_executor::ExecutionIndices;
    use narwhal_types::{Certificate, Header};
    use prometheus::Registry;

    #[tokio::test]
    pub async fn single_node_init() {
        let master_controller = ControllerRouter::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key,
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: vec![utils::new_network_address()],
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();

        let registry = Registry::default();
        let metrics = Arc::new(ValidatorMetrics::new(&registry));
        let validator = ValidatorState::new(public_key, secret, &genesis, &store_path, metrics);

        validator.halt_validator();
        validator.unhalt_validator();
    }

    fn create_test_validator() -> ValidatorState {
        let master_controller = ControllerRouter::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key,
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: vec![utils::new_network_address()],
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();
        let registry = Registry::default();
        let metrics = Arc::new(ValidatorMetrics::new(&registry));
        ValidatorState::new(public_key, secret, &genesis, &store_path, metrics)
    }

    fn create_test_execution_indices() -> ExecutionIndices {
        ExecutionIndices {
            next_certificate_index: 1,
            next_batch_index: 1,
            next_transaction_index: 1,
        }
    }

    fn create_test_consensus_output() -> ConsensusOutput {
        let dummy_header = Header::default();
        let dummy_certificate = Certificate {
            header: dummy_header,
            votes: Vec::new(),
        };
        ConsensusOutput {
            certificate: dummy_certificate,
            consensus_index: 1,
        }
    }

    #[allow(unused_must_use)]
    #[tokio::test]
    pub async fn process_create_asset_txn() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;
        let transaction = create_create_asset_transaction(sender_kp.public().clone(), 0, fee, recent_block_hash);
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();
    }

    #[allow(unused_must_use)]
    #[tokio::test]
    pub async fn process_payment_txn() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;
        let transaction = create_create_asset_transaction(sender_kp.public().clone(), 0, fee, recent_block_hash);
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();

        // create payment transaction
        const TEST_ASSET_ID: u64 = 0;
        const TEST_AMOUNT: u64 = 1000000;
        let receiver_kp = generate_production_keypair::<KeyPair>();
        let fee: u64 = 1000;
        let transaction = create_payment_transaction(
            sender_kp.public().clone(),
            receiver_kp.public(),
            TEST_ASSET_ID,
            TEST_AMOUNT,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();
    }

    #[allow(unused_must_use)]
    #[tokio::test]
    pub async fn process_create_orderbook_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;

        for asset_number in 0..5 {
            let transaction =
                create_create_asset_transaction(sender_kp.public().clone(), asset_number, fee, recent_block_hash);
            let signed_transaction = transaction.sign(&sender_kp).unwrap();
            let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

            validator
                .handle_consensus_transaction(
                    &dummy_consensus_output,
                    dummy_execution_indices.clone(),
                    consensus_transaction.clone(),
                )
                .await
                .unwrap();
        }

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;
        let transaction = create_create_orderbook_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();
    }

    #[allow(unused_must_use)]
    #[tokio::test]
    pub async fn process_place_limit_order_and_cancel_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;

        for asset_number in 0..5 {
            let transaction =
                create_create_asset_transaction(sender_kp.public().clone(), asset_number, fee, recent_block_hash);
            let signed_transaction = transaction.sign(&sender_kp).unwrap();
            let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

            validator
                .handle_consensus_transaction(
                    &dummy_consensus_output,
                    dummy_execution_indices.clone(),
                    consensus_transaction.clone(),
                )
                .await
                .unwrap();
        }

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;
        let transaction = create_create_orderbook_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();

        const TEST_PRICE: u64 = 100;
        const TEST_QUANTITY: u64 = 100;
        let fee: u64 = 1000;
        let transaction = create_limit_order_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            OrderSide::Bid as u64,
            TEST_PRICE,
            TEST_QUANTITY,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();

        // cancel order
        const TEST_ORDER_ID: u64 = 1;
        let fee: u64 = 1000;
        let transaction = create_cancel_order_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            OrderSide::Bid as u64,
            TEST_ORDER_ID,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();
    }

    #[allow(unused_must_use)]
    #[tokio::test]
    pub async fn process_place_limit_order_and_update_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;

        for asset_number in 0..5 {
            let transaction =
                create_create_asset_transaction(sender_kp.public().clone(), asset_number, fee, recent_block_hash);
            let signed_transaction = transaction.sign(&sender_kp).unwrap();
            let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

            validator
                .handle_consensus_transaction(
                    &dummy_consensus_output,
                    dummy_execution_indices.clone(),
                    consensus_transaction.clone(),
                )
                .await
                .unwrap();
        }

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee: u64 = 1000;
        let transaction = create_create_orderbook_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();

        // create limit order transaction
        const TEST_PRICE: u64 = 100;
        const TEST_QUANTITY: u64 = 100;
        let fee: u64 = 1000;
        let transaction = create_limit_order_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            OrderSide::Bid as u64,
            TEST_PRICE,
            TEST_QUANTITY,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();

        // cancel order
        const TEST_ORDER_ID: u64 = 1;
        let fee: u64 = 1000;
        let transaction = create_update_order_transaction(
            sender_kp.public().clone(),
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            OrderSide::Bid as u64,
            TEST_PRICE,
            TEST_QUANTITY,
            TEST_ORDER_ID,
            fee,
            recent_block_hash,
        );
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let consensus_transaction = ConsensusTransaction::new(&signed_transaction);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                consensus_transaction,
            )
            .await
            .unwrap();
    }
}
