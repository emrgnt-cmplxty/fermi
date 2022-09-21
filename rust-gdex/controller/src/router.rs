//! Master controller contains all relevant blockchain controllers
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::{
    bank::controller::BankController, consensus::controller::ConsensusController, controller::Controller,
    futures::controller::FuturesController, spot::controller::SpotController, stake::controller::StakeController,
};

// gdex

// mysten

// external
use tracing::{info};

// constants
const CATCHUP_STATE_FREQUENCY: u64 = 100;

use gdex_types::{
    error::GDEXError,
    store::{PostProcessStore},
    transaction::{Transaction},
};
// external
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ENUMS

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum ControllerType {
    Bank = 0,
    Stake = 1,
    Spot = 2,
    Consensus = 3,
    Futures = 4,
}

impl ControllerType {
    pub fn from_i32(value: i32) -> Result<Self, GDEXError> {
        match value {
            0 => Ok(ControllerType::Bank),
            1 => Ok(ControllerType::Stake),
            2 => Ok(ControllerType::Spot),
            3 => Ok(ControllerType::Consensus),
            4 => Ok(ControllerType::Futures),
            _ => Err(GDEXError::DeserializationError),
        }
    }
}

// INTERFACE

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControllerRouter {
    pub consensus_controller: Arc<Mutex<ConsensusController>>,
    pub bank_controller: Arc<Mutex<BankController>>,
    pub stake_controller: Arc<Mutex<StakeController>>,
    pub spot_controller: Arc<Mutex<SpotController>>,
    pub futures_controller: Arc<Mutex<FuturesController>>,
}

impl Default for ControllerRouter {
    fn default() -> Self {
        let bank_controller = Arc::new(Mutex::new(BankController::default()));
        let stake_controller = Arc::new(Mutex::new(StakeController::default()));
        let spot_controller = Arc::new(Mutex::new(SpotController::default()));
        let consensus_controller = Arc::new(Mutex::new(ConsensusController::default()));
        let futures_controller = Arc::new(Mutex::new(FuturesController::default()));

        Self {
            consensus_controller,
            bank_controller,
            stake_controller,
            spot_controller,
            futures_controller,
        }
    }
}

impl ControllerRouter {
    pub fn initialize_controllers(&self) {
        self.consensus_controller.lock().unwrap().initialize(self);
        self.bank_controller.lock().unwrap().initialize(self);
        self.stake_controller.lock().unwrap().initialize(self);
        self.spot_controller.lock().unwrap().initialize(self);
        self.futures_controller.lock().unwrap().initialize(self);
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
        match self.futures_controller.lock().unwrap().initialize_controller_account() {
            Ok(()) => (),
            Err(err) => panic!("Failed to initialize futures_controller account: {:?}", err),
        }
    }

    pub fn handle_consensus_transaction(&self, transaction: &Transaction) -> Result<(), GDEXError> {
        let target_controller = ControllerType::from_i32(transaction.target_controller)?;
        match target_controller {
            ControllerType::Consensus => {
                return self
                    .consensus_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction);
            }
            ControllerType::Bank => {
                return self
                    .bank_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction);
            }
            ControllerType::Stake => {
                return self
                    .stake_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction);
            }
            ControllerType::Spot => {
                return self
                    .spot_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction);
            }
            ControllerType::Futures => {
                return self
                    .futures_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction);
            }
        }
    }

    pub async fn create_catchup_state(&self, post_process_store: &PostProcessStore, block_number: u64) {
        if block_number % CATCHUP_STATE_FREQUENCY == 0 {
            let state = vec![
                ConsensusController::create_catchup_state(self.consensus_controller.clone(), block_number),
                BankController::create_catchup_state(self.bank_controller.clone(), block_number),
                StakeController::create_catchup_state(self.stake_controller.clone(), block_number),
                SpotController::create_catchup_state(self.spot_controller.clone(), block_number),
                FuturesController::create_catchup_state(self.futures_controller.clone(), block_number),
            ];
            let mut total_catchup_state: Vec<Vec<u8>> = Vec::new();

            // if serialization failure occurs do not save bad state
            if state.iter().filter(|x| x.is_err()).count() == 0 {
                for catchup_state in state {
                    total_catchup_state.push(catchup_state.unwrap());
                }
            }

            // print size for logging purposes
            let catchup_size: u64 = total_catchup_state.iter().map(|x| x.len() as u64).sum();
            info!(
                "Generating catchup snap at block {} of size {}",
                block_number, catchup_size
            );

            // store catchup state
            post_process_store
                .catchup_state_store
                .write(block_number, total_catchup_state)
                .await;
        }
    }

    pub async fn process_end_of_block(&self, post_process_store: &PostProcessStore, block_number: u64) {
        ConsensusController::process_end_of_block(self.consensus_controller.clone(), post_process_store, block_number)
            .await;
        BankController::process_end_of_block(self.bank_controller.clone(), post_process_store, block_number).await;
        StakeController::process_end_of_block(self.stake_controller.clone(), post_process_store, block_number).await;
        SpotController::process_end_of_block(self.spot_controller.clone(), post_process_store, block_number).await;
        FuturesController::process_end_of_block(self.futures_controller.clone(), post_process_store, block_number)
            .await;
    }
}
