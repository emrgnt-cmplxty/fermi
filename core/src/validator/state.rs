//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use super::genesis_state::ValidatorGenesisState;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use gdex_controller::master::MasterController;
use gdex_types::{
    account::ValidatorKeyPair,
    committee::{Committee, ValidatorName},
    error::GDEXError,
    transaction::{SignedTransaction, TransactionDigest, TransactionVariant},
};
use narwhal_config::{Committee as ConsensusCommittee};
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use narwhal_executor::{ExecutionIndices, ExecutionState};
use std::{
    collections::HashSet,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tracing::{info, trace};
use tokio::sync::mpsc::Sender;
/// Tracks recently submitted transactions to eventually implement transaction gating
// TODO - implement the gating and garbage collection
pub struct ValidatorStore {
    /// The transaction map tracks recently submitted transactions
    pub tranasaction_map: Mutex<HashSet<TransactionDigest>>,
}
impl Default for ValidatorStore {
    fn default() -> Self {
        Self {
            tranasaction_map: Mutex::new(HashSet::new()),
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
    /// A channel to tell consensus to reconfigure.
    tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
}

impl ValidatorState {
    // TODO: This function takes both committee and genesis as parameter.
    // Technically genesis already contains committee information. Could consider merging them.
    pub fn new(name: ValidatorName, secret: StableSyncValidatorSigner, genesis: &ValidatorGenesisState, tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>) -> Self {
        ValidatorState {
            name,
            secret,
            halted: AtomicBool::new(false),
            committee: ArcSwap::from(Arc::new(genesis.committee().unwrap())),
            master_controller: genesis.master_controller().clone(),
            validator_store: ValidatorStore::default(),
            tx_reconfigure_consensus
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
    pub async fn handle_transaction(&self, _transaction: &SignedTransaction) -> Result<(), GDEXError> {
        trace!("Handling a new transaction with the ValidatorState",);
        Ok(())
    }
}

#[async_trait]
impl ExecutionState for ValidatorState {
    type Transaction = SignedTransaction;
    type Error = GDEXError;
    type Outcome = Vec<u8>;

    async fn handle_consensus_transaction(
        &self,
        _consensus_output: &narwhal_consensus::ConsensusOutput,
        _execution_indices: ExecutionIndices,
        signed_transaction: Self::Transaction,
    ) -> Result<(Self::Outcome, Option<narwhal_config::Committee>), Self::Error> {
        let transaction = signed_transaction.get_transaction_payload();
        match transaction.get_variant() {
            TransactionVariant::PaymentTransaction(payment) => {
                self.master_controller.bank_controller.lock().unwrap().transfer(
                    transaction.get_sender(),
                    payment.get_receiver(),
                    payment.get_asset_id(),
                    payment.get_amount(),
                )?
            }
            TransactionVariant::CreateAssetTransaction(_create_asset) => self
                .master_controller
                .bank_controller
                .lock()
                .unwrap()
                .create_asset(transaction.get_sender())?,
        };

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

#[cfg(test)]
mod test_validator_state {
    use super::*;
    use crate::{builder::genesis_state::GenesisStateBuilder, genesis_ceremony::VALIDATOR_FUNDING_AMOUNT};
    use gdex_types::{
        account::ValidatorPubKeyBytes,
        crypto::{get_key_pair_from_rng, KeypairTraits, Signer},
        node::ValidatorInfo,
        transaction::SignedTransaction,
        utils,
    };
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::{generate_production_keypair, Hash, KeyPair, DIGEST_LEN};
    use narwhal_executor::ExecutionIndices;
    use narwhal_types::{BatchDigest, Certificate, Header};

    #[tokio::test]
    pub async fn single_node_init() {
        let master_controller = MasterController::default();
        let (tx_reconfigure_consensus, _rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);

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
        let validator = ValidatorState::new(public_key, secret, &genesis, tx_reconfigure_consensus);

        validator.halt_validator();
        validator.unhalt_validator();
    }

    #[tokio::test]
    pub async fn process_payment_txn() {
        let master_controller = MasterController::default();
        let (tx_reconfigure_consensus, _rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);

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
        let validator = ValidatorState::new(public_key, secret, &genesis, tx_reconfigure_consensus);

        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BatchDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = utils::create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        let dummy_execution_indices = ExecutionIndices {
            next_certificate_index: 1,
            next_batch_index: 1,
            next_transaction_index: 1,
        };
        let dummy_header = Header::default();
        let dummy_certificate = Certificate {
            header: dummy_header,
            votes: Vec::new(),
        };
        let dummy_consensus_output = ConsensusOutput {
            certificate: dummy_certificate,
            consensus_index: 1,
        };

        validator
            .handle_consensus_transaction(
                &dummy_consensus_output,
                dummy_execution_indices.clone(),
                signed_create_asset_txn,
            )
            .await
            .unwrap();

        // create payment transaction
        let receiver_kp = generate_production_keypair::<KeyPair>();
        let payment_txn = utils::create_payment_transaction(&sender_kp, &receiver_kp, 0, 1000000, recent_block_hash);
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
}
