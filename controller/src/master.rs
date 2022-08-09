//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::{bank::BankController, spot::SpotController, stake::StakeController};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterController {
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
            bank_controller: bank_controller_ref,
            stake_controller,
            spot_controller,
        }
    }
}
