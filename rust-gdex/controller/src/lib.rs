//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

pub mod utils;

pub mod controller;

pub mod bank;

pub mod consensus;

pub mod futures;

pub mod router;

pub mod spot;

pub mod stake;

#[cfg(any(test, feature = "testing"))]
pub trait ControllerTestBed {
    fn get_controller_router(&self) -> &router::ControllerRouter;

    fn generic_initialize(&self) {
        self.get_controller_router().initialize_controllers();
        self.get_controller_router().initialize_controller_accounts();
    }

    fn initialize(&self);
}
