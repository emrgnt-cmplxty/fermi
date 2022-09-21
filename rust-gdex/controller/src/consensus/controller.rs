//! consensus controller contains all relevant consensus params
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::controller::Controller;
use crate::router::ControllerRouter;

// gdex
use gdex_types::{
    account::AccountPubKey,
    crypto::ToFromBytes,
    error::GDEXError,
    store::PostProcessStore,
    transaction::{parse_request_type, Transaction},
};

// mysten

// external
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// CONSTANTS

pub const CONSENSUS_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"CONSENSUSCONTROLLERAAAAAAAAAAAAA";

const DEFAULT_BATCH_SIZE: usize = 500_000;
const DEFAULT_MAX_DELAY_MILLIS: u64 = 200; // .2 sec

// INTERFACE

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusController {
    controller_account: AccountPubKey,
    pub batch_size: usize,
    pub max_batch_delay: Duration,
}

impl Default for ConsensusController {
    fn default() -> Self {
        Self {
            controller_account: AccountPubKey::from_bytes(CONSENSUS_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            batch_size: DEFAULT_BATCH_SIZE,
            max_batch_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MILLIS),
        }
    }
}

#[async_trait]
impl Controller for ConsensusController {
    fn initialize(&mut self, _master_controller: &ControllerRouter) {}

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError> {
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError> {
        let request_type = parse_request_type(transaction.request_type)?;

        #[allow(clippy::match_single_binding)]
        match request_type {
            _ => Err(GDEXError::InvalidRequestTypeError),
        }
    }

    async fn process_end_of_block(
        _controller: Arc<Mutex<Self>>,
        _process_block_store: &PostProcessStore,
        _block_number: u64,
    ) {
    }

    fn create_catchup_state(controller: Arc<Mutex<Self>>, _block_number: u64) -> Result<Vec<u8>, GDEXError> {
        match bincode::serialize(&controller.lock().unwrap().clone()) {
            Ok(v) => Ok(v),
            Err(_) => Err(GDEXError::SerializationError),
        }
    }
}

#[cfg(test)]
pub mod consensus_tests {
    use super::*;
    
    #[test]
    fn create_consensus_catchup_state_default() {
        let consensus_controller = Arc::new(Mutex::new(ConsensusController::default()));
        let catchup_state = ConsensusController::create_catchup_state(consensus_controller, 0);
        assert!(catchup_state.is_ok());
        let catchup_state = catchup_state.unwrap();
        println!("Catchup state is {} bytes", catchup_state.len());

        match bincode::deserialize(&catchup_state) {
            Ok(ConsensusController { batch_size, ..}) => {
                assert_eq!(batch_size, DEFAULT_BATCH_SIZE);
            },
            Err(_) => panic!("deserializing catchup_state_default failed")
        }
    }
}