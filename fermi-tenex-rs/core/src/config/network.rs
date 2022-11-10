// fermi
use crate::{
    builder::network_config::NetworkConfigBuilder,
    config::{node::NodeConfig, Config, CONSENSUS_DB_NAME, GRPC_DB_NAME},
    validator::genesis_state::ValidatorGenesisState,
};
use fermi_types::{
    account::{AccountKeyPair, ValidatorKeyPair},
    committee::Committee,
    crypto::get_key_pair_from_rng,
    node::ValidatorInfo,
    serialization::KeyPairBase64,
    utils,
};
// external
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

/// Configures the network communications by specifying all validator node configs, account keys, and the genesis state
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub validator_configs: Vec<NodeConfig>,
    #[serde_as(as = "Vec<KeyPairBase64>")]
    pub account_keys: Vec<AccountKeyPair>,
    pub genesis: ValidatorGenesisState,
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

    pub fn generate_with_rng(config_dir: &Path, quorum_size: usize) -> Self {
        NetworkConfigBuilder::new(config_dir)
            .committee_size(NonZeroUsize::new(quorum_size).unwrap())
            .build()
    }

    pub fn generate(config_dir: &Path, quorum_size: usize) -> Self {
        Self::generate_with_rng(config_dir, quorum_size)
    }

    /// Generate a fullnode config based on this `NetworkConfig`. This is useful if you want to run
    /// a fullnode and have it connect to a network defined by this `NetworkConfig`.
    pub fn generate_fullnode_config(&self) -> NodeConfig {
        let key_pair: Arc<ValidatorKeyPair> =
            Arc::new(get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut OsRng));
        let validator_config = &self.validator_configs[0];

        let mut db_path = validator_config.consensus_db_path.clone();
        db_path.pop();

        NodeConfig {
            key_pair,
            consensus_db_path: db_path.join(CONSENSUS_DB_NAME),
            grpc_db_path: db_path.join(GRPC_DB_NAME),
            metrics_address: utils::new_network_address(),
            admin_interface_port: utils::get_available_port(),
            json_rpc_address: utils::new_network_address(),
            websocket_address: Some(utils::available_local_socket_address()),
            consensus_config: None,
            enable_event_processing: true,
            enable_gossip: true,
            enable_reconfig: false,
            genesis: validator_config.genesis.clone(),
        }
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;
    use crate::config::{genesis::GenesisConfig, GDEX_NETWORK_CONFIG};

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
        let _validator_configs = network_config.validator_configs();
        let _conf_validator_set = network_config.validator_set();
        let _committee = network_config.committee();
        let _into_validator_configs = network_config.into_validator_configs();
    }

    #[test]
    pub fn config_random() {
        let temp_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_dir.path();
        let fermi_config_dir = working_dir.join(GDEX_NETWORK_CONFIG);
        let _config = super::NetworkConfig::generate(fermi_config_dir.as_path(), 5);
    }
}
