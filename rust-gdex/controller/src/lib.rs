//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

pub mod utils;

pub mod controller;

pub mod event_manager;

pub mod bank;

pub mod consensus;

pub mod futures;

pub mod router;

pub mod spot;

pub mod stake;

use std::sync::{Arc, Mutex};
#[cfg(any(test, feature = "testing"))]
pub trait ControllerTestBed {
    fn get_controller_router(&self) -> Arc<Mutex<router::ControllerRouter>>;

    fn generic_initialize(&self) {
        self.get_controller_router().lock().unwrap().initialize_controllers();
        self.get_controller_router()
            .lock()
            .unwrap()
            .initialize_controller_accounts();
    }

    fn initialize(&self);
}
