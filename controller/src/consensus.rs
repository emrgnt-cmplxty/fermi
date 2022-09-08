//! consensus controller contains all relevant consensus params
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::controller::Controller;
use crate::master::MasterController;

// gdex
use gdex_types::{
    account::AccountPubKey,
    crypto::ToFromBytes,
    error::GDEXError,
    transaction::{parse_request_type, Transaction},
};

// mysten

// external
use serde::{Deserialize, Serialize};
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

impl Controller for ConsensusController {
    fn initialize(&mut self, _master_controller: &MasterController) {}

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

    fn post_process(&mut self, _block_number: u64) {}
}
