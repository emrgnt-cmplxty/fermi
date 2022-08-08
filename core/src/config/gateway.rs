//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/gateway.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use super::Config;
use gdex_types::{committee::EpochId, node::ValidatorInfo};
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct GatewayConfig {
    pub epoch: EpochId,
    pub validator_set: Vec<ValidatorInfo>,
    pub send_timeout: Duration,
    pub recv_timeout: Duration,
    pub buffer_size: usize,
    pub db_folder_path: PathBuf,
}

impl Config for GatewayConfig {}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            epoch: 0,
            validator_set: vec![],
            send_timeout: Duration::from_micros(4000000),
            recv_timeout: Duration::from_micros(4000000),
            buffer_size: 650000,
            db_folder_path: Default::default(),
        }
    }
}
