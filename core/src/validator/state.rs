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
use narwhal_executor::{ExecutionIndices, ExecutionState};
use narwhal_types::{CertificateDigest, SequenceNumber};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tracing::{debug, info};

/// Tracks recently submitted transactions to eventually implement transaction gating
pub struct ValidatorStore {
    /// The transaction map tracks recently submitted transactions
    transaction_cache: Mutex<HashMap<TransactionDigest, CertificateDigest>>,
    certificate_cache: Mutex<HashMap<CertificateDigest, SequenceNumber>>,
    // garbage collection depth
    gc_depth: u64,
}

impl Default for ValidatorStore {
    fn default() -> Self {
        Self {
            transaction_cache: Mutex::new(HashMap::new()),
            certificate_cache: Mutex::new(HashMap::new()),
            gc_depth: 50,
        }
    }
}

impl ValidatorStore {
    pub fn check_seen_transaction(&self, transaction: &Transaction) -> bool {
        let transaction_digest = transaction.digest();
        return self.transaction_cache.lock().unwrap().contains_key(&transaction_digest);
    }

    pub fn check_seen_certificate_digest(&self, certificate_digest: &CertificateDigest) -> bool {
        return self.certificate_cache.lock().unwrap().contains_key(certificate_digest);
    }

    pub fn insert_unconfirmed_transaction(&self, transaction: &Transaction) {
        let transaction_digest = transaction.digest();
        self.transaction_cache.lock().unwrap().insert(
            transaction_digest,
            CertificateDigest::new([0; 32]), // Insert with dummy certificate, which will later be overwritten
        );
    }

    pub fn insert_confirmed_transaction(&self, transaction: &Transaction, consensus_output: &ConsensusOutput) {
        let transaction_digest = transaction.digest();
        let certificate_digest = consensus_output.certificate.digest();
        let mut locked_certificate_cache = self.certificate_cache.lock().unwrap();
        let max_seq_num_so_far = locked_certificate_cache.values().max();

        let is_new_seq_num =
            max_seq_num_so_far.is_none() || consensus_output.consensus_index > *max_seq_num_so_far.unwrap();

        self.transaction_cache
            .lock()
            .unwrap()
            .insert(transaction_digest, certificate_digest);
        locked_certificate_cache.insert(certificate_digest, consensus_output.consensus_index);
        drop(locked_certificate_cache);
        if is_new_seq_num {
            self.handle_new_sequence_number();
        }
    }

    fn handle_new_sequence_number(&self) {
        self.prune()
        // extend this
    }

    fn prune(&self) {
        let mut locked_certificate_cache = self.certificate_cache.lock().unwrap();
        if locked_certificate_cache.len() > self.gc_depth as usize {
            let mut threshold = locked_certificate_cache.values().max().unwrap() - self.gc_depth;
            let dummy_certificate_digest = &mut CertificateDigest::new([0; 32]);
            locked_certificate_cache.retain(|_k, v| v > &mut threshold);
            self.transaction_cache
                .lock()
                .unwrap()
                .retain(|_k, v| v == dummy_certificate_digest || locked_certificate_cache.contains_key(v));
        }
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
    pub async fn new(name: ValidatorName, secret: StableSyncValidatorSigner, genesis: &ValidatorGenesisState) -> Self {
        ValidatorState {
            name,
            secret,
            halted: AtomicBool::new(false),
            committee: ArcSwap::from(Arc::new(genesis.committee().unwrap())),
            master_controller: genesis.master_controller().clone(),
            validator_store: ValidatorStore::default(),
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
        debug!("Handling a new pre-consensus transaction with the ValidatorState",);
        self.validator_store
            .insert_unconfirmed_transaction(&transaction.get_transaction_payload());
        Ok(())
    }
}

#[cfg(test)]
mod test_validator_state {
    use super::*;
    use crate::{builder::genesis_state::GenesisStateBuilder, genesis_ceremony::VALIDATOR_FUNDING_AMOUNT};
    use gdex_types::{
        account::ValidatorPubKeyBytes,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        utils,
    };

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
        let validator = ValidatorState::new(public_key, secret, &genesis).await;

        validator.halt_validator();
        validator.unhalt_validator();
    }
}

#[async_trait]
impl ExecutionState for ValidatorState {
    type Transaction = SignedTransaction;
    type Error = GDEXError;
    type Outcome = Vec<u8>;

    async fn handle_consensus_transaction(
        &self,
        consensus_output: &narwhal_consensus::ConsensusOutput,
        _execution_indices: ExecutionIndices,
        transaction: Self::Transaction,
    ) -> Result<(Self::Outcome, Option<narwhal_config::Committee>), Self::Error> {
        debug!(
            "Processing transaction = {:?} with consensus output = {:?}",
            transaction, consensus_output
        );

        self.validator_store
            .insert_confirmed_transaction(transaction.get_transaction_payload(), consensus_output);

        Ok((Vec::default(), None))
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
