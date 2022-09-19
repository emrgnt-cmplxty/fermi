//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

pub mod controller;

pub mod bank;

pub mod consensus;

pub mod futures;

pub mod main_controller;

pub mod spot;

pub mod stake;

#[cfg(test)]
pub trait ControllerTestBed {
    fn get_main_controller(&self) -> &main_controller::MainController;

    fn generic_initialize(&self) {
        self.get_main_controller().initialize_controllers();
        self.get_main_controller().initialize_controller_accounts();
    }

    fn initialize(&self);
}
