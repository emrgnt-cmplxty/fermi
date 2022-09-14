//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/node.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::{
    config::{consensus::ConsensusConfig, Config, Genesis},
    validator::genesis_state::ValidatorGenesisState,
};
use anyhow::Result;
use gdex_types::account::{ValidatorKeyPair, ValidatorPubKeyBytes};
use gdex_types::crypto::GDEXAddress;
use gdex_types::crypto::KeypairTraits;
use gdex_types::serialization::KeyPairBase64;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configures external connection parameters for a given validator
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeConfig {
    #[serde(default = "default_key_pair")]
    #[serde_as(as = "Arc<KeyPairBase64>")]
    pub key_pair: Arc<ValidatorKeyPair>,
    pub consensus_db_path: PathBuf,
    pub gdex_db_path: PathBuf,
    #[serde(default = "default_json_rpc_address")]
    pub json_rpc_address: SocketAddr,
    #[serde(default = "default_websocket_address")]
    pub websocket_address: Option<SocketAddr>,

    #[serde(default = "default_metrics_address")]
    pub metrics_address: SocketAddr,
    #[serde(default = "default_admin_interface_port")]
    pub admin_interface_port: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_config: Option<ConsensusConfig>,

    #[serde(default)]
    pub enable_event_processing: bool,

    #[serde(default)]
    pub enable_gossip: bool,

    #[serde(default)]
    pub enable_reconfig: bool,

    pub genesis: Genesis,
}

fn default_key_pair() -> Arc<ValidatorKeyPair> {
    Arc::new(gdex_types::crypto::get_random_key_pair())
}

fn default_metrics_address() -> SocketAddr {
    use std::net::{IpAddr, Ipv4Addr};
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 9184)
}

pub fn default_admin_interface_port() -> u16 {
    1337
}

pub fn default_json_rpc_address() -> SocketAddr {
    use std::net::{IpAddr, Ipv4Addr};
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 9000)
}

pub fn default_websocket_address() -> Option<SocketAddr> {
    use std::net::{IpAddr, Ipv4Addr};
    Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 9001))
}

impl Config for NodeConfig {}

impl NodeConfig {
    pub fn key_pair(&self) -> &ValidatorKeyPair {
        &self.key_pair
    }

    pub fn public_key(&self) -> ValidatorPubKeyBytes {
        self.key_pair.public().into()
    }

    pub fn gdex_address(&self) -> GDEXAddress {
        (&self.public_key()).into()
    }

    pub fn consensus_db_path(&self) -> &Path {
        &self.consensus_db_path
    }

    pub fn gdex_db_path(&self) -> &Path {
        &self.gdex_db_path
    }

    pub fn consensus_config(&self) -> Option<&ConsensusConfig> {
        self.consensus_config.as_ref()
    }

    pub fn genesis(&self) -> Result<&ValidatorGenesisState> {
        self.genesis.genesis()
    }
}

#[cfg(test)]
mod node_tests {
    use crate::{
        builder::network_config::NetworkConfigBuilder,
        config::{genesis::GenesisConfig, GDEX_NETWORK_CONFIG},
    };
    use std::num::NonZeroUsize;

    #[test]
    pub fn config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_dir.path();
        let gdex_config_dir = working_dir.join(GDEX_NETWORK_CONFIG);
        let genesis_conf = GenesisConfig::for_local_testing();

        let network_config = NetworkConfigBuilder::new(gdex_config_dir)
            .committee_size(NonZeroUsize::new(genesis_conf.committee_size).unwrap())
            .initial_accounts_config(genesis_conf)
            .build();
        let validator_config = network_config.validator_configs()[0].clone();

        let _default_key_pair = validator_config.key_pair();
        let _public_key = validator_config.public_key();
        let _gdex_address = validator_config.gdex_address();
        let _consensus_db_path = validator_config.consensus_db_path();
        let _gdex_db_path = validator_config.gdex_db_path();
        let _consensus_config = validator_config.consensus_config();
        let _genesis = validator_config.genesis();
    }
}
