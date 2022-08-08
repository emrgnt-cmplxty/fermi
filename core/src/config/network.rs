//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/swarm.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use super::{genesis, node::NodeConfig, Config, FULL_NODE_DB_PATH};
use crate::builder::config::ConfigBuilder;
use gdex_types::{
    account::{AccountKeyPair, ValidatorKeyPair},
    committee::Committee,
    crypto::get_key_pair_from_rng,
    node::ValidatorInfo,
    serialization::KeyPairBase64,
    utils,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

/// This is a config that is used for testing or local use as it contains the config and keys for
/// all validators
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub validator_configs: Vec<NodeConfig>,
    #[serde_as(as = "Vec<KeyPairBase64>")]
    pub account_keys: Vec<AccountKeyPair>,
    pub genesis: genesis::Genesis,
}

impl Config for NetworkConfig {}

impl NetworkConfig {
    pub fn validator_configs(&self) -> &[NodeConfig] {
        &self.validator_configs
    }

    pub fn validator_set(&self) -> &[ValidatorInfo] {
        self.genesis.validator_set()
    }

    pub fn committee(&self) -> Committee {
        self.genesis.committee().unwrap()
    }

    pub fn into_validator_configs(self) -> Vec<NodeConfig> {
        self.validator_configs
    }

    pub fn generate_with_rng<R: rand::CryptoRng + rand::RngCore>(
        config_dir: &Path,
        quorum_size: usize,
        rng: R,
    ) -> Self {
        ConfigBuilder::new(config_dir)
            .committee_size(NonZeroUsize::new(quorum_size).unwrap())
            .rng(rng)
            .build()
    }

    pub fn generate(config_dir: &Path, quorum_size: usize) -> Self {
        Self::generate_with_rng(config_dir, quorum_size, OsRng)
    }

    /// Generate a fullnode config based on this `NetworkConfig`. This is useful if you want to run
    /// a fullnode and have it connect to a network defined by this `NetworkConfig`.
    pub fn generate_fullnode_config(&self) -> NodeConfig {
        let key_pair: Arc<ValidatorKeyPair> = Arc::new(get_key_pair_from_rng(&mut OsRng).1);
        let validator_config = &self.validator_configs[0];

        let mut db_path = validator_config.db_path.clone();
        db_path.pop();

        NodeConfig {
            key_pair,
            db_path: db_path.join(FULL_NODE_DB_PATH),
            network_address: utils::new_network_address(),
            metrics_address: utils::available_local_socket_address(),
            admin_interface_port: utils::get_available_port(),
            json_rpc_address: utils::available_local_socket_address(),
            websocket_address: Some(utils::available_local_socket_address()),
            consensus_config: None,
            enable_event_processing: true,
            enable_gossip: true,
            enable_reconfig: false,
            genesis: validator_config.genesis.clone(),
        }
    }
}
