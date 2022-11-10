use crate::{
    config::{consensus::ConsensusConfig, Config, Genesis},
    validator::genesis_state::ValidatorGenesisState,
};
use fermi_types::{
    account::{ValidatorKeyPair, ValidatorPubKeyBytes},
    crypto::{GDEXAddress, KeypairTraits},
    serialization::KeyPairBase64,
};
// external
use anyhow::Result;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Configures external connection parameters for a given validator
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeConfig {
    #[serde(default = "default_key_pair")]
    #[serde_as(as = "Arc<KeyPairBase64>")]
    pub key_pair: Arc<ValidatorKeyPair>,
    pub consensus_db_path: PathBuf,
    pub grpc_db_path: PathBuf,
    #[serde(default = "default_websocket_address")]
    pub websocket_address: Option<SocketAddr>,

    #[serde(default = "default_json_rpc_address")]
    pub json_rpc_address: Multiaddr,
    #[serde(default = "default_metrics_address")]
    pub metrics_address: Multiaddr,
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
    Arc::new(fermi_types::crypto::get_random_key_pair())
}

fn default_metrics_address() -> Multiaddr {
    "/ip4/127.0.0.1/tcp/9184".parse().unwrap()
}

pub fn default_admin_interface_port() -> u16 {
    1337
}

pub fn default_json_rpc_address() -> Multiaddr {
    "/ip4/127.0.0.1/tcp/9185".parse().unwrap()
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

    pub fn fermi_address(&self) -> GDEXAddress {
        (&self.public_key()).into()
    }

    pub fn consensus_db_path(&self) -> &Path {
        &self.consensus_db_path
    }

    pub fn grpc_db_path(&self) -> &Path {
        &self.grpc_db_path
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
        let fermi_config_dir = working_dir.join(GDEX_NETWORK_CONFIG);
        let genesis_conf = GenesisConfig::for_local_testing();

        let network_config = NetworkConfigBuilder::new(fermi_config_dir)
            .committee_size(NonZeroUsize::new(genesis_conf.committee_size).unwrap())
            .initial_accounts_config(genesis_conf)
            .build();
        let validator_config = network_config.validator_configs()[0].clone();

        let _default_key_pair = validator_config.key_pair();
        let _public_key = validator_config.public_key();
        let _fermi_address = validator_config.fermi_address();
        let _consensus_db_path = validator_config.consensus_db_path();
        let _grpc_db_path = validator_config.grpc_db_path();
        let _consensus_config = validator_config.consensus_config();
        let _genesis = validator_config.genesis();
    }
}
