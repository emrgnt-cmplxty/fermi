// IMPORTS

// crate
use crate::event_manager::EventEmitter;
use crate::router::ControllerRouter;

// fermi
use fermi_types::{
    error::GDEXError,
    store::{CriticalPathStore, RPCStore, RPCStoreHandle},
    transaction::Transaction,
};

// external
use async_trait::async_trait;
use serde::Serialize;
use std::sync::{Arc, Mutex};

// TRAIT
#[async_trait]
pub trait Controller<RPCImpl>: Clone + Serialize + EventEmitter
where
    RPCImpl: Send + Sync + 'static,
{
    fn initialize(&mut self, controller_router: &ControllerRouter);

    fn initialize_controller_account(&self) -> Result<(), GDEXError>;

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;

    fn critical_process_end_of_block(&self, _critical_path_store: &CriticalPathStore, _block_number: u64) {}

    fn non_critical_process_end_of_block(&self, _rpc_store: &RPCStore, _block_number: u64) {}

    fn get_catchup_state(&self) -> Result<Vec<u8>, GDEXError> {
        match bincode::serialize(&self.clone()) {
            Ok(v) => Ok(v),
            Err(_) => Err(GDEXError::SerializationError),
        }
    }

    fn rpc_is_implemented() -> bool {
        false
    }

    // Fetching data off the router is a blocking operation
    // Therefore, it is recommended that the router is separate from
    // The critical path of consensus, e.g. in a separate thread
    fn generate_json_rpc_module(
        _controller_router: Arc<Mutex<ControllerRouter>>,
        _generate_json_rpc_module: Arc<RPCStoreHandle>,
    ) -> Result<jsonrpsee::RpcModule<RPCImpl>, GDEXError> {
        Err(GDEXError::NotImplemented)
    }
}
