// IMPORTS

// crate
use crate::master::MasterController;

// gdex
use gdex_types::{error::GDEXError, new_transaction::NewTransaction};

// TRAIT

pub trait Controller {
    fn initialize(&mut self, master_controller: &MasterController);

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError>;

    fn handle_consensus_transaction(&mut self, transaction: &NewTransaction) -> Result<(), GDEXError>;
}
