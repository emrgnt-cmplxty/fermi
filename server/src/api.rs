// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
#[cfg(not(tarpaulin))]
#[allow(clippy::all)]
mod validator {
    include!(concat!(env!("OUT_DIR"), "/gdex.validator.ValidatorAPI.rs"));
}

pub use validator::{
    validator_a_p_i_client::ValidatorAPIClient,
    validator_a_p_i_server::{ValidatorAPI, ValidatorAPIServer},
};
