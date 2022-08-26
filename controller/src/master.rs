//! Master controller contains all relevant blockchain controllers
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::{
    bank::BankController, consensus::ConsensusController, controller::Controller, spot::SpotController,
    stake::StakeController,
};

// gdex
use gdex_types::{error::GDEXError, transaction::Transaction};

// mysten

// external
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// INTERFACE

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterController {
    pub consensus_controller: Arc<Mutex<ConsensusController>>,
    pub bank_controller: Arc<Mutex<BankController>>,
    pub stake_controller: Arc<Mutex<StakeController>>,
    pub spot_controller: Arc<Mutex<SpotController>>,
}

impl Default for MasterController {
    fn default() -> Self {
        let bank_controller = Arc::new(Mutex::new(BankController::default()));
        let stake_controller = Arc::new(Mutex::new(StakeController::default()));
        let spot_controller = Arc::new(Mutex::new(SpotController::default()));
        let consensus_controller = Arc::new(Mutex::new(ConsensusController::default()));

        Self {
            consensus_controller,
            bank_controller,
            stake_controller,
            spot_controller,
        }
    }
}

impl MasterController {
    pub fn initialize_controllers(&self) {
        self.consensus_controller.lock().unwrap().initialize(self);
        self.bank_controller.lock().unwrap().initialize(self);
        self.stake_controller.lock().unwrap().initialize(self);
        self.spot_controller.lock().unwrap().initialize(self);
    }
}

pub trait HandleConsensus {
    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;
}
