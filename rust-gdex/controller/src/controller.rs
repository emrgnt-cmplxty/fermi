// IMPORTS

// crate
use crate::router::ControllerRouter;

// gdex
use gdex_types::{error::GDEXError, store::PostProcessStore, transaction::Transaction};

// external
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

// TRAIT
#[async_trait]
pub trait Controller {
    fn initialize(&mut self, master_controller: &ControllerRouter);

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError>;

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;

    async fn process_end_of_block(
        controller: Arc<Mutex<Self>>,
        _process_block_store: &PostProcessStore,
        block_number: u64,
    );

    fn create_catchup_state(controller: Arc<Mutex<Self>>, block_number: u64) -> Result<Vec<u8>, GDEXError>;
}
