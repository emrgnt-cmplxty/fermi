// IMPORTS

use crate::master::MasterController;

pub trait Controller {
    fn initialize(&mut self, master_controller: &MasterController);
}
