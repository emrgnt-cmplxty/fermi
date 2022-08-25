//! Master controller contains all relevant blockchain controllers
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::{bank::BankController, consensus::ConsensusController, spot::SpotController, stake::StakeController};
use gdex_types::{
    transaction::{Transaction},
    error::{GDEXError}
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Duration
};

const DEFAULT_BATCH_SIZE: usize = 500_000;
const DEFAULT_MAX_DELAY_MILLIS: u64 = 200; // .2 sec

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterController {
    pub consensus_controller: ConsensusController,
    pub bank_controller: Arc<Mutex<BankController>>,
    pub stake_controller: StakeController,
    pub spot_controller: Arc<Mutex<SpotController>>
}

impl Default for MasterController {
    fn default() -> Self {
        let bank_controller = Arc::new(Mutex::new(BankController::default()));
        let stake_controller = StakeController::new(Arc::clone(&bank_controller));
        let spot_controller = Arc::new(Mutex::new(SpotController::new(Arc::clone(&bank_controller))));

        Self {
            consensus_controller: ConsensusController {
                batch_size: DEFAULT_BATCH_SIZE,
                max_batch_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MILLIS),
            },
            bank_controller,
            stake_controller,
            spot_controller
        }
    }
}

// handle consensus trait for all controllers
pub trait HandleConsensus {
    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;
}