//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use super::genesis_state::ValidatorGenesisState;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use gdex_controller::master::MasterController;
use gdex_types::transaction::Transaction;
use gdex_types::{
    account::ValidatorKeyPair,
    committee::{Committee, ValidatorName},
    error::GDEXError,
    transaction::{SignedTransaction, TransactionDigest},
};
use narwhal_consensus::ConsensusOutput;
use narwhal_crypto::Hash;
use narwhal_executor::{ExecutionIndices, ExecutionState, SerializedTransaction};
use mysten_store::{
    reopen,
    rocks::{open_cf, DBMap},
    Store,
};
use narwhal_types::{CertificateDigest, SequenceNumber};
use std::path::PathBuf;
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tracing::{info, trace};

/// Tracks recently submitted transactions to implement transaction gating
pub struct ValidatorStore {
    /// The transaction map tracks recently submitted transactions
    transaction_cache: Mutex<HashMap<TransactionDigest, Option<CertificateDigest>>>,
    certificate_cache: Mutex<HashMap<CertificateDigest, SequenceNumber>>,
    // garbage collection depth
    gc_depth: u64,
    pub transaction_store: Store<SequenceNumber, Vec<SerializedTransaction>>,
    pub sequence_store: Store<SequenceNumber, CertificateDigest>,
}

impl ValidatorStore {
    const TRANSACTIONS_CF: &'static str = "transactions";
    const SEQUENCE_CF: &'static str = "sequence";

    fn new(store_path: &PathBuf) -> Self {
        Self::reopen(store_path)
    }

    pub fn reopen<Path: AsRef<std::path::Path>>(store_path: Path) -> Self {
        let rocksdb =
            open_cf(store_path, None, &[Self::TRANSACTIONS_CF, Self::SEQUENCE_CF]).expect("Cannot open database");

        let (transactions_map, sequence_map) = reopen!(&rocksdb,
            Self::TRANSACTIONS_CF;<SequenceNumber, Vec<SerializedTransaction>>,
            Self::SEQUENCE_CF;<SequenceNumber, CertificateDigest>
        );

        let transaction_store = Store::new(transactions_map);
        let sequence_store = Store::new(sequence_map);

        Self {
            transaction_cache: Mutex::new(HashMap::new()),
            certificate_cache: Mutex::new(HashMap::new()),
            gc_depth: 50,
            transaction_store,
            sequence_store,
        }
    }

    pub fn contains_transaction(&self, transaction: &Transaction) -> bool {
        let transaction_digest = transaction.digest();
        return self.transaction_cache.lock().unwrap().contains_key(&transaction_digest);
    }

    pub fn contains_certificate_digest(&self, certificate_digest: &CertificateDigest) -> bool {
        return self.certificate_cache.lock().unwrap().contains_key(certificate_digest);
    }

    pub fn insert_unconfirmed_transaction(&self, transaction: &Transaction) {
        let transaction_digest = transaction.digest();
        self.transaction_cache.lock().unwrap().insert(
            transaction_digest,
            None, // Insert with dummy certificate, which will later be overwritten
        );
    }

    pub fn insert_confirmed_transaction(&self, transaction: &Transaction, consensus_output: &ConsensusOutput) {
        let transaction_digest = transaction.digest();
        let certificate_digest = consensus_output.certificate.digest();
        let mut locked_certificate_cache = self.certificate_cache.lock().unwrap();
        let max_seq_num_so_far = locked_certificate_cache.values().max();

        let _is_new_seq_num =
            max_seq_num_so_far.is_none() || consensus_output.consensus_index > *max_seq_num_so_far.unwrap();

        self.transaction_cache
            .lock()
            .unwrap()
            .insert(transaction_digest, Some(certificate_digest));
        locked_certificate_cache.insert(certificate_digest, consensus_output.consensus_index);
    }

    pub fn prune(&self) {
        let mut locked_certificate_cache = self.certificate_cache.lock().unwrap();
        if locked_certificate_cache.len() > self.gc_depth as usize {
            let mut threshold = locked_certificate_cache.values().max().unwrap() - self.gc_depth;
            locked_certificate_cache.retain(|_k, v| v > &mut threshold);
            self.transaction_cache
                .lock()
                .unwrap()
                .retain(|_k, v| v.is_none() || locked_certificate_cache.contains_key(&v.unwrap()));
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
    // A map of transactions which have been seen
    pub validator_store: ValidatorStore,
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
            validator_store: ValidatorStore::new(store_db_path),
        }
    }

    pub fn halt_validator(&self) {
        self.halted.store(true, Ordering::Relaxed);
    }

    pub fn unhalt_validator(&self) {
        self.halted.store(false, Ordering::Relaxed);
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
        let transaction = signed_transaction.get_transaction_payload();
        // match transaction.get_variant() {
        //     TransactionVariant::PaymentTransaction(payment) => {
        //         self.master_controller.bank_controller.lock().unwrap().transfer(
        //             transaction.get_sender(),
        //             payment.get_receiver(),
        //             payment.get_asset_id(),
        //             payment.get_amount(),
        //         )?
        //     }
        //     TransactionVariant::CreateAssetTransaction(_create_asset) => self
        //         .master_controller
        //         .bank_controller
        //         .lock()
        //         .unwrap()
        //         .create_asset(transaction.get_sender())?,
        //     TransactionVariant::CreateOrderbookTransaction(orderbook) => self
        //         .master_controller
        //         .spot_controller
        //         .lock()
        //         .unwrap()
        //         .create_orderbook(orderbook.get_base_asset_id(), orderbook.get_quote_asset_id())?,
        //     TransactionVariant::PlaceOrderTransaction(order) => {
        //         match order {
        //             OrderRequest::Market {
        //                 base_asset_id,
        //                 quote_asset_id,
        //                 side,
        //                 quantity,
        //                 local_timestamp: _ts,
        //             } => {
        //                 dbg!(base_asset_id, quote_asset_id, side, quantity);
        //             }
        //             OrderRequest::Limit {
        //                 base_asset_id,
        //                 quote_asset_id,
        //                 side,
        //                 price,
        //                 quantity,
        //                 local_timestamp: _ts,
        //             } => {
        //                 // TODO: find out why these u64 are references
        //                 self.master_controller
        //                     .spot_controller
        //                     .lock()
        //                     .unwrap()
        //                     .place_limit_order(
        //                         *base_asset_id,
        //                         *quote_asset_id,
        //                         transaction.get_sender(),
        //                         *side,
        //                         *quantity,
        //                         *price,
        //                     )?
        //             }
        //             OrderRequest::CancelOrder { id, side } => {
        //                 dbg!(id, side);
        //             }
        //             OrderRequest::Update {
        //                 id,
        //                 side,
        //                 price,
        //                 quantity,
        //                 local_timestamp: _ts,
        //             } => {
        //                 dbg!(id, side, price, quantity);
        //             }
        //         }
        //     }
        // };

        self.validator_store
            .insert_confirmed_transaction(transaction, consensus_output);

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
    use crate::{builder::genesis_state::GenesisStateBuilder, genesis_ceremony::VALIDATOR_FUNDING_AMOUNT};
    use gdex_types::{
        account::ValidatorPubKeyBytes,
        crypto::{get_key_pair_from_rng, KeypairTraits, Signer},
        node::ValidatorInfo,
        order_book::OrderSide,
        transaction::{
            create_asset_creation_transaction, create_orderbook_creation_transaction, create_payment_transaction,
            create_place_limit_order_transaction, SignedTransaction,
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

        let key: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: public_key.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
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

        let key: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: public_key.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
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

        validator
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
        let recent_block_hash = CertificateDigest::new([0; DIGEST_LEN]);
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
        let recent_block_hash = CertificateDigest::new([0; DIGEST_LEN]);
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
        let recent_block_hash = CertificateDigest::new([0; DIGEST_LEN]);
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

        //dbg!(validator.master_controller.bank_controller.lock().unwrap().get_num_assets());

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = CertificateDigest::new([0; DIGEST_LEN]);
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
        //dbg!(validator.master_controller.spot_controller.lock().unwrap().get_orderbook(TEST_BASE_ASSET_ID, TEST_QUOTE_ASSET_ID).unwrap());
    }

    #[tokio::test]
    pub async fn process_place_limit_order_transaction() {
        let validator: ValidatorState = create_test_validator();
        let dummy_consensus_output = create_test_consensus_output();
        let dummy_execution_indices = create_test_execution_indices();

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = CertificateDigest::new([0; DIGEST_LEN]);
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

        //dbg!(validator.master_controller.bank_controller.lock().unwrap().get_num_assets());

        // create orderbook transaction
        const TEST_BASE_ASSET_ID: u64 = 1;
        const TEST_QUOTE_ASSET_ID: u64 = 2;
        let recent_block_hash = CertificateDigest::new([0; DIGEST_LEN]);
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
    }
}
