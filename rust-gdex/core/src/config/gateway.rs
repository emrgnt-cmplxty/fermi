//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/gateway.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::config::Config;
use gdex_types::{committee::EpochId, node::ValidatorInfo};
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_SEND_TIMEOUT: u64 = 4000000;
const DEFAULT_RECEIVE_TIMEOUT: u64 = 4000000;
const DEFAULT_BUFFER_SIZE: usize = 650000;

/// Configures the network gateway for the local validator
#[derive(Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Latest epoch of consensus
    pub epoch: EpochId,
    /// Current validators
    pub validator_set: Vec<ValidatorInfo>,
    /// Timeout for sending transactions
    pub send_timeout: Duration,
    /// Timeout for receiving transactions
    pub recv_timeout: Duration,
    /// Timeout for receiving transactions
    pub buffer_size: usize,
    pub db_folder_path: PathBuf,
}

impl Config for GatewayConfig {}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            epoch: 0,
            validator_set: vec![],
            send_timeout: Duration::from_micros(DEFAULT_SEND_TIMEOUT),
            recv_timeout: Duration::from_micros(DEFAULT_RECEIVE_TIMEOUT),
            buffer_size: DEFAULT_BUFFER_SIZE,
            db_folder_path: Default::default(),
        }
    }
}
