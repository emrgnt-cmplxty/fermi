//! consensus controller contains all relevant consensus params
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::controller::Controller;
use crate::event_manager::{EventEmitter, EventManager};
use crate::router::ControllerRouter;

// gdex
use gdex_types::{
    account::AccountPubKey, crypto::ToFromBytes, error::GDEXError, store::ProcessBlockStore, transaction::Transaction,
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
impl Controller for ConsensusController {
    fn initialize(&mut self, controller_router: &ControllerRouter) {
        self.event_manager = Arc::clone(&controller_router.event_manager);
    }

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError> {
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, _transaction: &Transaction) -> Result<(), GDEXError> {
        Err(GDEXError::InvalidRequestTypeError)
    }

    async fn process_end_of_block(
        _controller: Arc<Mutex<Self>>,
        _process_block_store: &ProcessBlockStore,
        _block_number: u64,
    ) {
    }
}

impl EventEmitter for ConsensusController {
    fn get_event_manager(&mut self) -> &mut Arc<Mutex<EventManager>> {
        &mut self.event_manager
    }
}
