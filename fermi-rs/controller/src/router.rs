// IMPORTS

// crate
use crate::{
    bank::controller::BankController, consensus::controller::ConsensusController, controller::Controller,
    event_manager::EventManager, futures::controller::FuturesController, spot::controller::SpotController,
    stake::controller::StakeController,
};

// fermi
use fermi_types::{
    error::GDEXError,
    store::{CatchupState, CriticalPathStore, RPCStore, RPCStoreHandle},
    transaction::{ExecutionEvents, Transaction},
};

// mysten

// external
use jsonrpsee::RpcModule;

// constants
const CATCHUP_STATE_FREQUENCY: u64 = 100;

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
    // state
    pub event_manager: Arc<Mutex<EventManager>>,
    // controllers
    pub consensus_controller: Arc<Mutex<ConsensusController>>,
    pub bank_controller: Arc<Mutex<BankController>>,
    pub stake_controller: Arc<Mutex<StakeController>>,
    pub spot_controller: Arc<Mutex<SpotController>>,
    pub futures_controller: Arc<Mutex<FuturesController>>,
}

impl Default for ControllerRouter {
    fn default() -> Self {
        // state
        let event_manager = Arc::new(Mutex::new(EventManager::new()));
        // controllers
        let bank_controller = Arc::new(Mutex::new(BankController::default()));
        let stake_controller = Arc::new(Mutex::new(StakeController::default()));
        let spot_controller = Arc::new(Mutex::new(SpotController::default()));
        let consensus_controller = Arc::new(Mutex::new(ConsensusController::default()));
        let futures_controller = Arc::new(Mutex::new(FuturesController::default()));

        Self {
            // state
            event_manager,
            // controllers
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

    // TODO - Change the return signature of this to "ExecutionEvents" and roll the error into ExecutionEvents
    // This will also remove the need for the type ExecutionResult
    pub fn handle_consensus_transaction(&self, transaction: &Transaction) -> Result<ExecutionEvents, GDEXError> {
        // reset execution result
        // TODO - https://github.com/fermiorg/fermi/issues/176 - elimintate the need for reset by more properly handling errors
        // eventually we wont need this as we will have strong guarantees that this
        // val gets rest on each iteration but for now we can fail halfway though (in theory) so must be reset
        self.event_manager.lock().unwrap().reset();

        let target_controller = ControllerType::from_i32(transaction.target_controller)?;
        match target_controller {
            ControllerType::Consensus => {
                self.consensus_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction)?;
            }
            ControllerType::Bank => {
                self.bank_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction)?;
            }
            ControllerType::Stake => {
                self.stake_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction)?;
            }
            ControllerType::Spot => {
                self.spot_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction)?;
            }
            ControllerType::Futures => {
                self.futures_controller
                    .lock()
                    .unwrap()
                    .handle_consensus_transaction(transaction)?;
            }
        }
        Ok(self.event_manager.lock().unwrap().emit())
    }

    pub fn critical_process_end_of_block(
        &self,
        critical_path_store: &CriticalPathStore,
        block_number: u64,
    ) -> Result<(), GDEXError> {
        self.consensus_controller
            .lock()
            .unwrap()
            .critical_process_end_of_block(critical_path_store, block_number);

        self.bank_controller
            .lock()
            .unwrap()
            .critical_process_end_of_block(critical_path_store, block_number);

        self.stake_controller
            .lock()
            .unwrap()
            .critical_process_end_of_block(critical_path_store, block_number);

        self.spot_controller
            .lock()
            .unwrap()
            .critical_process_end_of_block(critical_path_store, block_number);

        self.futures_controller
            .lock()
            .unwrap()
            .critical_process_end_of_block(critical_path_store, block_number);
        Ok(())
    }

    // Same todo previously mentioned, need to move away from using non-async mutex's for controller related functionality
    pub fn non_critical_process_end_of_block(&self, rpc_store: &RPCStore, block_number: u64) -> Result<(), GDEXError> {
        if block_number % CATCHUP_STATE_FREQUENCY == 0 {
            // TODO - It is quite gross and potentially error prone to just stick the catch-up states into
            // A vector and then write them to the store. We should probably have a more structured way of
            // doing this. Moreover, it should fit into the Controller workflow more directly.
            let consensus_controller_state = self.consensus_controller.lock().unwrap().get_catchup_state()?;
            let bank_controller_state = self.bank_controller.lock().unwrap().get_catchup_state()?;
            let stake_controller_state = self.stake_controller.lock().unwrap().get_catchup_state()?;
            let spot_controller_state = self.spot_controller.lock().unwrap().get_catchup_state()?;
            let futures_controller_state = self.futures_controller.lock().unwrap().get_catchup_state()?;

            let state = vec![
                consensus_controller_state,
                bank_controller_state,
                stake_controller_state,
                spot_controller_state,
                futures_controller_state,
            ];

            rpc_store.catchup_state_store.try_write(0, CatchupState { state });
        }

        self.consensus_controller
            .lock()
            .unwrap()
            .non_critical_process_end_of_block(rpc_store, block_number);

        self.bank_controller
            .lock()
            .unwrap()
            .non_critical_process_end_of_block(rpc_store, block_number);

        self.consensus_controller
            .lock()
            .unwrap()
            .non_critical_process_end_of_block(rpc_store, block_number);

        self.stake_controller
            .lock()
            .unwrap()
            .non_critical_process_end_of_block(rpc_store, block_number);

        self.spot_controller
            .lock()
            .unwrap()
            .non_critical_process_end_of_block(rpc_store, block_number);

        self.futures_controller
            .lock()
            .unwrap()
            .non_critical_process_end_of_block(rpc_store, block_number);

        Ok(())
    }

    pub fn generate_rpc_module(
        controller_router: Arc<Mutex<ControllerRouter>>,
        rpc_store_handle: Arc<RPCStoreHandle>,
    ) -> RpcModule<()> {
        let mut module = RpcModule::new(());

        // note - we intentionally use unwraps inside this function because we want the program to panic if the rpc module fails to initialize

        if ConsensusController::rpc_is_implemented() {
            module
                .merge(
                    ConsensusController::generate_json_rpc_module(
                        Arc::clone(&controller_router),
                        Arc::clone(&rpc_store_handle),
                    )
                    .unwrap(),
                )
                .unwrap()
        }

        if BankController::rpc_is_implemented() {
            module
                .merge(
                    BankController::generate_json_rpc_module(
                        Arc::clone(&controller_router),
                        Arc::clone(&rpc_store_handle),
                    )
                    .unwrap(),
                )
                .unwrap()
        }

        if StakeController::rpc_is_implemented() {
            module
                .merge(
                    StakeController::generate_json_rpc_module(
                        Arc::clone(&controller_router),
                        Arc::clone(&rpc_store_handle),
                    )
                    .unwrap(),
                )
                .unwrap()
        }

        if SpotController::rpc_is_implemented() {
            module
                .merge(
                    SpotController::generate_json_rpc_module(
                        Arc::clone(&controller_router),
                        Arc::clone(&rpc_store_handle),
                    )
                    .unwrap(),
                )
                .unwrap()
        }

        if FuturesController::rpc_is_implemented() {
            module
                .merge(
                    FuturesController::generate_json_rpc_module(
                        Arc::clone(&controller_router),
                        Arc::clone(&rpc_store_handle),
                    )
                    .unwrap(),
                )
                .unwrap()
        }
        module
    }
}
