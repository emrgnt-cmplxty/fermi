// IMPORTS

// crate
use crate::main_controller::MainController;

// gdex
use gdex_types::{error::GDEXError, store::ProcessBlockStore, transaction::Transaction};

// external
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

// TRAIT
#[async_trait]
pub trait Controller {
    fn initialize(&mut self, master_controller: &MainController);

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError>;

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;

    async fn process_end_of_block(
        controller: Arc<Mutex<Self>>,
        _process_block_store: &ProcessBlockStore,
        block_number: u64,
    );
}
