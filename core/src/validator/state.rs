use crate::config::genesis::Genesis;
use arc_swap::ArcSwap;
use gdex_proc::master::MasterController;
use gdex_types::transaction::SignedTransaction;
use gdex_types::{
    account::ValidatorKeyPair,
    committee::{Committee, ValidatorName},
    error::GDEXError,
    transaction::TransactionDigest,
};
use std::{
    collections::HashSet,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub struct ValidatorStore {
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

/// The validator state encapsulates all state, drives execution, and ensures safety.
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
    /// Controller of various blockchain modules
    pub master_controller: MasterController,
    // A map of transactions which have been seen
    pub validator_store: ValidatorStore,
}

impl ValidatorState {
    // TODO: This function takes both committee and genesis as parameter.
    // Technically genesis already contains committee information. Could consider merging them.
    pub async fn new(name: ValidatorName, secret: StableSyncValidatorSigner, genesis: &Genesis) -> Self {
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
    pub async fn handle_transaction(&self, transaction: SignedTransaction) -> Result<(), GDEXError> {
        Ok(())
        // self.metrics.tx_orders.inc();
        // // Check the sender's signature.
        // transaction.verify().map_err(|e| {
        //     self.metrics.signature_errors.inc();
        //     e
        // })?;
        // let transaction_digest = *transaction.digest();

        // let response = self.handle_transaction_impl(transaction).await;
        // match response {
        //     Ok(r) => Ok(()),
        //     // If we see an error, it is possible that a certificate has already been processed.
        //     // In that case, we could still return Ok to avoid showing confusing errors.
        //     Err(err) => {
        //         if self.database.effects_exists(&transaction_digest)? {
        //             Ok(())
        //         } else {
        //             Err(err)
        //         }
        //     }
        // }
    }
}

#[cfg(test)]
mod test_validator {
    use super::*;
    use crate::config::genesis::Builder;
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
            stake: 1,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let builder = Builder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let validator = ValidatorState::new(public_key, secret, &genesis).await;

        validator.halt_validator();
        validator.unhalt_validator();
    }
}
