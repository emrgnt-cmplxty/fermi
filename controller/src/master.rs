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

// mysten

use gdex_types::{error::GDEXError, transaction::Transaction};
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

    pub fn initialize_controller_accounts(&self) {
        match self
            .consensus_controller
            .lock()
            .unwrap()
            .initialize_controller_account()
        {
            Ok(()) => (),
            Err(err) => panic!("Failed to initialize consensus_controller account: {:?}", err),
        }
        match self.bank_controller.lock().unwrap().initialize_controller_account() {
            Ok(()) => (),
            Err(err) => panic!("Failed to initialize bank_controller account: {:?}", err),
        }
        match self.stake_controller.lock().unwrap().initialize_controller_account() {
            Ok(()) => (),
            Err(err) => panic!("Failed to initialize stake_controller account: {:?}", err),
        }
        match self.spot_controller.lock().unwrap().initialize_controller_account() {
            Ok(()) => (),
            Err(err) => panic!("Failed to initialize spot_controller account: {:?}", err),
        }
    }

    pub fn handle_consensus_transaction(&self, transaction: &Transaction) -> Result<(), GDEXError> {
        self.consensus_controller
            .lock()
            .unwrap()
            .handle_consensus_transaction(transaction)?;

        self.bank_controller
            .lock()
            .unwrap()
            .handle_consensus_transaction(transaction)?;

        self.stake_controller
            .lock()
            .unwrap()
            .handle_consensus_transaction(transaction)?;

        self.spot_controller
            .lock()
            .unwrap()
            .handle_consensus_transaction(transaction)?;

        Ok(())
    }

    pub fn post_process(&self, block_number: u64) {
        self.consensus_controller.lock().unwrap().post_process(block_number);

        self.bank_controller.lock().unwrap().post_process(block_number);

        self.stake_controller.lock().unwrap().post_process(block_number);

        self.spot_controller.lock().unwrap().post_process(block_number);
    }
}
