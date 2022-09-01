//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use super::genesis_state::ValidatorGenesisState;
use crate::validator::metrics::ValidatorMetrics;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use gdex_controller::master::MasterController;
use gdex_types::transaction::Transaction;
use gdex_types::{
    account::ValidatorKeyPair,
    block::{Block, BlockCertificate, BlockDigest, BlockInfo, BlockNumber},
    committee::{Committee, ValidatorName},
    error::GDEXError,
    transaction::{SignedTransaction, TransactionDigest},
};
use mysten_store::{
    reopen,
    rocks::{open_cf, DBMap},
    Map, Store,
};
use narwhal_consensus::ConsensusOutput;
use narwhal_crypto::Hash;
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
};
use tracing::{info, trace};
/// Tracks recently submitted transactions to implement transaction gating
pub struct ValidatorStore {
    /// The transaction map tracks recently submitted transactions
    transaction_cache: Mutex<HashMap<TransactionDigest, Option<BlockDigest>>>,
    block_digest_cache: Mutex<HashMap<BlockDigest, BlockNumber>>,
    // garbage collection depth
    gc_depth: u64,
    pub block_store: Store<BlockNumber, Block>,
    pub block_info_store: Store<BlockNumber, BlockInfo>,
    pub block_number: AtomicU64,
    // singleton store that keeps only the most recent block info at key 0
    pub last_block_info_store: Store<u64, BlockInfo>,
}

impl ValidatorStore {
    const BLOCKS_CF: &'static str = "blocks";
    const BLOCK_INFO_CF: &'static str = "block_info";
    const LAST_BLOCK_CF: &'static str = "last_block";

    pub fn reopen<Path: AsRef<std::path::Path>>(store_path: Path) -> Self {
        let rocksdb = open_cf(
            store_path,
            None,
            &[Self::BLOCKS_CF, Self::BLOCK_INFO_CF, Self::LAST_BLOCK_CF],
        )
        .expect("Cannot open database");

        let (block_map, block_info_map, last_block_map) = reopen!(&rocksdb,
            Self::BLOCKS_CF;<BlockNumber, Block>,
            Self::BLOCK_INFO_CF;<BlockNumber, BlockInfo>,
            Self::LAST_BLOCK_CF;<u64, BlockInfo>
        );

        let block_number_from_dbmap = last_block_map.get(&0_u64);

        let block_store = Store::new(block_map);
        let block_info_store = Store::new(block_info_map);
        let last_block_info_store = Store::new(last_block_map);

        // TODO load the state if last block is not 0, i.e. not at genesis
        let block_number = match block_number_from_dbmap {
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
            block_store,
            block_info_store,
            block_number: AtomicU64::new(block_number),
            last_block_info_store,
        }
    }

    pub fn cache_contains_transaction(&self, transaction: &Transaction) -> bool {
        let transaction_digest = transaction.digest();
        return self.transaction_cache.lock().unwrap().contains_key(&transaction_digest);
    }

    pub fn cache_contains_block_digest(&self, block_digest: &BlockDigest) -> bool {
        return self.block_digest_cache.lock().unwrap().contains_key(block_digest);
    }

    pub fn insert_unconfirmed_transaction(&self, transaction: &Transaction) {
        let transaction_digest = transaction.digest();
        self.transaction_cache.lock().unwrap().insert(
            transaction_digest,
            None, // Insert with no block digest, a digest will be added after confirmation
        );
    }

    pub fn insert_confirmed_transaction(&self, transaction: &Transaction, consensus_output: &ConsensusOutput) {
        let transaction_digest = transaction.digest();
        let block_digest = consensus_output.certificate.digest();
        let mut locked_block_digest_cache = self.block_digest_cache.lock().unwrap();
        let max_seq_num_so_far = locked_block_digest_cache.values().max();

        let _is_new_seq_num =
            max_seq_num_so_far.is_none() || consensus_output.consensus_index > *max_seq_num_so_far.unwrap();

        self.transaction_cache
            .lock()
            .unwrap()
            .insert(transaction_digest, Some(block_digest));
        locked_block_digest_cache.insert(block_digest, consensus_output.consensus_index);
    }

    pub async fn write_latest_block(
        &self,
        block_certificate: BlockCertificate,
        transactions: Vec<SerializedTransaction>,
    ) {
        // TODO - is there a way to acquire a mutable reference to the block-number without demanding &mut self?
        // this would allow us to avoid separate commands to load and add to the counter

        let block_number = self.block_number.load(std::sync::atomic::Ordering::SeqCst);
        let block_digest = block_certificate.digest();

        let block = Block {
            block_certificate: block_certificate.clone(),
            transactions,
        };

        let block_info = BlockInfo {
            block_number,
            block_digest,
        };

        // write-out the block information to associated stores
        self.block_store.write(block_number, block.clone()).await;
        self.block_info_store.write(block_number, block_info.clone()).await;
        self.last_block_info_store.write(0, block_info).await;
        // update the block number
        self.block_number.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
    pub master_controller: MasterController,
    /// A map of transactions which have been seen
    pub validator_store: ValidatorStore,
    /// Metrics around blockchain operations
    pub metrics: ValidatorMetrics,
}

impl ValidatorState {
    // TODO: This function takes both committee and genesis as parameter.
    // Technically genesis already contains committee information. Could consider merging them.
    pub fn new(
        name: ValidatorName,
        secret: StableSyncValidatorSigner,
        genesis: &ValidatorGenesisState,
        store_db_path: &PathBuf,
    ) -> Self {
        ValidatorState {
            name,
            secret,
            halted: AtomicBool::new(false),
            committee: ArcSwap::from(Arc::new(genesis.committee().unwrap())),
            master_controller: genesis.master_controller().clone(),
            validator_store: ValidatorStore::reopen(store_db_path),
            metrics: ValidatorMetrics::default(),
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
    pub fn handle_pre_consensus_transaction(&self, transaction: &SignedTransaction) -> Result<(), GDEXError> {
        trace!("Handling a new pre-consensus transaction with the ValidatorState",);
        self.validator_store
            .insert_unconfirmed_transaction(transaction.get_transaction_payload());
        Ok(())
    }
}

#[async_trait]
impl ExecutionState for ValidatorState {
    type Transaction = SignedTransaction;
    type Error = GDEXError;
    type Outcome = (ConsensusOutput, ExecutionIndices);

    async fn handle_consensus_transaction(
        &self,
        consensus_output: &narwhal_consensus::ConsensusOutput,
        execution_indices: ExecutionIndices,
        signed_transaction: Self::Transaction,
    ) -> Result<Self::Outcome, Self::Error> {
        self.metrics.increment_num_transactions_consensus();
        let transaction = signed_transaction.get_transaction_payload();

        self.validator_store
            .insert_confirmed_transaction(transaction, consensus_output);

        let result = self.master_controller.handle_consensus_transaction(transaction);

        if result.is_err() {
            self.metrics.increment_num_transactions_consensus_failed();
            return Err(result.unwrap_err());
        }
        Ok((consensus_output.clone(), execution_indices))
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
}

#[cfg(test)]
mod test_validator_state {
    use super::*;
    use crate::{
        builder::genesis_state::GenesisStateBuilder,
        genesis_ceremony::{VALIDATOR_BALANCE, VALIDATOR_FUNDING_AMOUNT},
    };
    use gdex_types::{
        account::ValidatorPubKeyBytes,
        crypto::{get_key_pair_from_rng, KeypairTraits, Signer},
        node::ValidatorInfo,
        order_book::OrderSide,
        transaction::{
            create_asset_creation_transaction, create_orderbook_creation_transaction, create_payment_transaction,
            create_place_cancel_order_transaction, create_place_limit_order_transaction,
            create_place_update_order_transaction, SignedTransaction,
        },
        utils,
    };
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::{generate_production_keypair, Hash, KeyPair, DIGEST_LEN};
    use narwhal_executor::ExecutionIndices;
    use narwhal_types::{Certificate, Header};

    #[tokio::test]
    pub async fn single_node_init() {
        let master_controller = MasterController::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key,
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();
        let validator = ValidatorState::new(public_key, secret, &genesis, &store_path);

        validator.halt_validator();
        validator.unhalt_validator();
    }

    fn create_test_validator() -> ValidatorState {
        let master_controller = MasterController::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key,
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();

        ValidatorState::new(public_key, secret, &genesis, &store_path)
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

    #[tokio::test]
    pub async fn process_create_asset_txn() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_create_asset_txn,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn process_payment_txn() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_create_asset_txn,
            )
            .await
            .unwrap();

        // create payment transaction
        const TEST_ASSET_ID: u64 = 0;
        const TEST_AMOUNT: u64 = 1000000;
        let receiver_kp = generate_production_keypair::<KeyPair>();
        let payment_txn =
            create_payment_transaction(&sender_kp, &receiver_kp, TEST_ASSET_ID, TEST_AMOUNT, recent_block_hash);
        let signed_digest = sender_kp.sign(&payment_txn.digest().get_array()[..]);
        let signed_payment_txn = SignedTransaction::new(sender_kp.public().clone(), payment_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_payment_txn,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn process_create_orderbook_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        for _ in 0..5 {
            validator
                .handle_consensus_transaction(
                    &dummy_consensus_output,
                    dummy_execution_indices.clone(),
                    signed_create_asset_txn.clone(),
                )
                .await
                .unwrap();
        }

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_orderbook_txn = create_orderbook_creation_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&create_orderbook_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_orderbook_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_create_asset_txn,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn process_place_limit_order_and_cancel_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        for _ in 0..5 {
            validator
                .handle_consensus_transaction(
                    &dummy_consensus_output,
                    dummy_execution_indices.clone(),
                    signed_create_asset_txn.clone(),
                )
                .await
                .unwrap();
        }

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_orderbook_txn = create_orderbook_creation_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&create_orderbook_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_orderbook_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_create_asset_txn,
            )
            .await
            .unwrap();

        const TEST_PRICE: u64 = 100;
        const TEST_QUANTITY: u64 = 100;
        let place_limit_order_txn = create_place_limit_order_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            OrderSide::Bid,
            TEST_PRICE,
            TEST_QUANTITY,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&place_limit_order_txn.digest().get_array()[..]);
        let signed_place_limit_order_txn =
            SignedTransaction::new(sender_kp.public().clone(), place_limit_order_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_place_limit_order_txn,
            )
            .await
            .unwrap();

        // cancel order
        const TEST_ORDER_ID: u64 = 1;
        let cancel_order_txn = create_place_cancel_order_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            TEST_ORDER_ID,
            OrderSide::Bid,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&cancel_order_txn.digest().get_array()[..]);
        let signed_cancel_order_txn =
            SignedTransaction::new(sender_kp.public().clone(), cancel_order_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_cancel_order_txn,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn process_place_limit_order_and_update_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        for _ in 0..5 {
            validator
                .handle_consensus_transaction(
                    &dummy_consensus_output,
                    dummy_execution_indices.clone(),
                    signed_create_asset_txn.clone(),
                )
                .await
                .unwrap();
        }

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_orderbook_txn = create_orderbook_creation_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&create_orderbook_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_orderbook_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_create_asset_txn,
            )
            .await
            .unwrap();

        const TEST_PRICE: u64 = 100;
        const TEST_QUANTITY: u64 = 100;
        let place_limit_order_txn = create_place_limit_order_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            OrderSide::Bid,
            TEST_PRICE,
            TEST_QUANTITY,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&place_limit_order_txn.digest().get_array()[..]);
        let signed_place_limit_order_txn =
            SignedTransaction::new(sender_kp.public().clone(), place_limit_order_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_place_limit_order_txn,
            )
            .await
            .unwrap();

        // cancel order
        const TEST_ORDER_ID: u64 = 1;
        let update_order_txn = create_place_update_order_transaction(
            &sender_kp,
            TEST_BASE_ASSET_ID,
            TEST_QUOTE_ASSET_ID,
            TEST_ORDER_ID,
            OrderSide::Bid,
            TEST_PRICE,
            TEST_QUANTITY + 1,
            recent_block_hash,
        );
        let signed_digest = sender_kp.sign(&update_order_txn.digest().get_array()[..]);
        let signed_update_order_txn =
            SignedTransaction::new(sender_kp.public().clone(), update_order_txn, signed_digest);

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_update_order_txn,
            )
            .await
            .unwrap();
    }
}
