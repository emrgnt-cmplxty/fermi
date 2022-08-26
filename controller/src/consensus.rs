//! consensus controller contains all relevant consensus params
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::controller::Controller;
use crate::master::MasterController;

// gdex
use gdex_types::{account::AccountPubKey, crypto::ToFromBytes, error::GDEXError, transaction::Transaction};

// mysten

// external
use serde::{Deserialize, Serialize};
use std::time::Duration;

// CONSTANTS

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
            controller_account: AccountPubKey::from_bytes(b"CONSENSUSCONTROLLERAAAAAAAAAAAAA").unwrap(),
            batch_size: DEFAULT_BATCH_SIZE,
            max_batch_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MILLIS),
        }
    }
}

impl Controller for ConsensusController {
    fn initialize(&mut self, _master_controller: &MasterController) -> Result<(), GDEXError> {
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, _transaction: &Transaction) -> Result<(), GDEXError> {
        Ok(())
    }
}
