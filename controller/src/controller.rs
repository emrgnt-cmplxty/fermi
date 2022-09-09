// IMPORTS

// crate
use crate::master::MasterController;
use async_trait::async_trait;
// gdex
use gdex_types::{error::GDEXError, store::ProcessBlockStore, transaction::Transaction};

// TRAIT
#[async_trait]
pub trait Controller {
    fn initialize(&mut self, master_controller: &MasterController);

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError>;

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;

    async fn process_end_of_block(&mut self, _process_block_store: &ProcessBlockStore, block_number: u64);
}
