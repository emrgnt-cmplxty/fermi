//! Master controller contains all relevant blockchain controllers
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::{bank::BankController, consensus::ConsensusController, spot::SpotController, stake::StakeController};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

const DEFAULT_MIN_BATCH_SIZE: usize = 1_000;
const DEFAULT_MAX_DELAY_MILLIS: u64 = 5_000; // 5 sec

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterController {
    pub consensus_controller: ConsensusController,
    pub bank_controller: Arc<Mutex<BankController>>,
    pub stake_controller: StakeController,
    pub spot_controller: SpotController,
}

impl Default for MasterController {
    fn default() -> Self {
        let bank_controller = BankController::default();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));
        let stake_controller = StakeController::new(Arc::clone(&bank_controller_ref));
        let spot_controller = SpotController::new(Arc::clone(&bank_controller_ref));
        Self {
            consensus_controller: ConsensusController {
                min_batch_size: DEFAULT_MIN_BATCH_SIZE,
                max_batch_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MILLIS),
            },
            bank_controller: bank_controller_ref,
            stake_controller,
            spot_controller,
        }
    }
}
