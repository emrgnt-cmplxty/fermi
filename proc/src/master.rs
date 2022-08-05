//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
use crate::{bank::BankController, spot::SpotController, stake::StakeController};
use core::cell::RefCell;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterController {
    pub bank_controller: Rc<RefCell<BankController>>,
    pub stake_controller: StakeController,
    pub spot_controller: SpotController,
}

impl Default for MasterController {
    fn default() -> Self {
        let bank_controller = BankController::default();
        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));
        let stake_controller = StakeController::new(Rc::clone(&bank_controller_ref));
        let spot_controller = SpotController::new(Rc::clone(&bank_controller_ref));
        Self {
            bank_controller: bank_controller_ref,
            stake_controller,
            spot_controller,
        }
    }
}
