//! consensus controller contains all relevant consensus params
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::{
    consensus::rpc_server::UnimplementedRPC,
    controller::Controller,
    event_manager::{EventEmitter, EventManager},
    router::ControllerRouter,
};
// gdex
use gdex_types::{account::AccountPubKey, crypto::ToFromBytes, error::GDEXError, transaction::Transaction};
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
    // controller state
    controller_account: AccountPubKey,
    pub batch_size: usize,
    pub max_batch_delay: Duration,
    // shared
    event_manager: Arc<Mutex<EventManager>>,
}

impl Default for ConsensusController {
    fn default() -> Self {
        Self {
            controller_account: AccountPubKey::from_bytes(CONSENSUS_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            batch_size: DEFAULT_BATCH_SIZE,
            max_batch_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MILLIS),
            // shared state
            event_manager: Arc::new(Mutex::new(EventManager::new())), // TEMPORARY
        }
    }
}

#[async_trait]
impl Controller<UnimplementedRPC> for ConsensusController {
    fn initialize(&mut self, controller_router: &ControllerRouter) {
        self.event_manager = Arc::clone(&controller_router.event_manager);
    }

    fn initialize_controller_account(&self) -> Result<(), GDEXError> {
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, _transaction: &Transaction) -> Result<(), GDEXError> {
        Err(GDEXError::InvalidRequestTypeError)
    }
}

#[cfg(test)]
pub mod consensus_tests {
    use super::*;

    #[test]
    fn create_consensus_catchup_state_default() {
        let consensus_controller = ConsensusController::default();
        let catchup_state = consensus_controller.get_catchup_state();
        assert!(catchup_state.is_ok());
        let catchup_state = catchup_state.unwrap();
        println!("Catchup state is {} bytes", catchup_state.len());

        match bincode::deserialize(&catchup_state) {
            Ok(ConsensusController { batch_size, .. }) => {
                assert_eq!(batch_size, DEFAULT_BATCH_SIZE);
            }
            Err(_) => panic!("deserializing catchup_state_default failed"),
        }
    }
}

impl EventEmitter for ConsensusController {
    fn get_event_manager(&mut self) -> &mut Arc<Mutex<EventManager>> {
        &mut self.event_manager
    }
}
