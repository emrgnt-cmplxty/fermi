// IMPORTS

// crate
use crate::master::MasterController;

// gdex
use gdex_types::{error::GDEXError, transaction::Transaction};

// TRAIT

pub trait Controller {
    fn initialize(&mut self, master_controller: &MasterController);

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError>;
}
