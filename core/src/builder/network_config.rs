//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/builder.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::{
    builder::genesis_state::GenesisStateBuilder,
    config::{
        consensus::ConsensusConfig,
        genesis::{GenesisConfig, ValidatorGenesisStateInfo},
        network::NetworkConfig,
        node::NodeConfig,
        {AUTHORITIES_DB_NAME, CONSENSUS_DB_NAME, DEFAULT_STAKE},
    },
};
use gdex_types::{
    account::{ValidatorKeyPair, ValidatorPubKeyBytes},
    crypto::{get_key_pair_from_rng, KeypairTraits},
    node::ValidatorInfo,
    utils,
};
use rand::rngs::OsRng;
use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
};

/// A config builder class which is used in the genesis process to generate a NetworkConfig
pub struct NetworkConfigBuilder<R = OsRng> {
    /// Associated random number generator
    rng: R,
    /// Directory of created config
    config_directory: PathBuf,
    /// Boolean parameter to determine port generation process, currently always set to True
    randomize_ports: bool,
    /// Size of committee
    committee_size: NonZeroUsize,
    /// Optional initial accounts configuration
    initial_accounts_config: Option<GenesisConfig>,
}

impl NetworkConfigBuilder {
    pub fn new<P: AsRef<Path>>(config_directory: P) -> Self {
        Self {
            rng: OsRng,
            config_directory: config_directory.as_ref().into(),
            randomize_ports: true,
            committee_size: NonZeroUsize::new(1).unwrap(),
            initial_accounts_config: None,
        }
    }
}

impl<R> NetworkConfigBuilder<R> {
    /// Set the randomize the ports and return a new object
    pub fn randomize_ports(mut self, randomize_ports: bool) -> Self {
        self.randomize_ports = randomize_ports;
        self
    }

    /// Set the committee size and return a new object
    pub fn committee_size(mut self, committee_size: NonZeroUsize) -> Self {
        self.committee_size = committee_size;
        self
    }

    /// Set initial accounts config and return a new object
    pub fn initial_accounts_config(mut self, initial_accounts_config: GenesisConfig) -> Self {
        self.initial_accounts_config = Some(initial_accounts_config);
        self
    }

    /// Set the rng and return a new object
    pub fn rng<N: ::rand::RngCore + ::rand::CryptoRng>(self, rng: N) -> NetworkConfigBuilder<N> {
        NetworkConfigBuilder {
            rng,
            config_directory: self.config_directory,
            randomize_ports: self.randomize_ports,
            committee_size: self.committee_size,
            initial_accounts_config: self.initial_accounts_config,
        }
    }
}

impl<R: ::rand::RngCore + ::rand::CryptoRng> NetworkConfigBuilder<R> {
    //TODO right now we always randomize ports, we may want to have a default port configuration
    /// Build a config with random validator inputs the networking ports
    /// are randomly selected via utils::new_network_address
    pub fn build(mut self) -> NetworkConfig {
        let validators = (0..self.committee_size.get())
            .map(|_| get_key_pair_from_rng(&mut self.rng).1)
            .map(|key_pair: ValidatorKeyPair| ValidatorGenesisStateInfo {
                key_pair,
                network_address: utils::new_network_address(),
                stake: DEFAULT_STAKE,
                narwhal_primary_to_primary: utils::new_network_address(),
                narwhal_worker_to_primary: utils::new_network_address(),
                narwhal_primary_to_worker: utils::new_network_address(),
                narwhal_worker_to_worker: utils::new_network_address(),
                narwhal_consensus_address: utils::new_network_address(),
            })
            .collect::<Vec<_>>();

        self.build_with_validators(validators)
    }

    /// Given a set of validators this returns a network config
    pub fn build_with_validators(mut self, validators: Vec<ValidatorGenesisStateInfo>) -> NetworkConfig {
        let validator_set = validators
            .iter()
            .enumerate()
            .map(|(i, validator)| {
                let name = format!("validator-{i}");
                let public_key: ValidatorPubKeyBytes = validator.key_pair.public().into();
                let stake = validator.stake;
                let network_address = validator.network_address.clone();

                ValidatorInfo {
                    name,
                    public_key,
                    stake,
                    delegation: 0, // no delegation yet at genesis
                    network_address,
                    narwhal_primary_to_primary: validator.narwhal_primary_to_primary.clone(),
                    narwhal_worker_to_primary: validator.narwhal_worker_to_primary.clone(),
                    narwhal_primary_to_worker: validator.narwhal_primary_to_worker.clone(),
                    narwhal_worker_to_worker: validator.narwhal_worker_to_worker.clone(),
                    narwhal_consensus_address: validator.narwhal_consensus_address.clone(),
                }
            })
            .collect::<Vec<_>>();

        let initial_accounts_config = self
            .initial_accounts_config
            .unwrap_or_else(GenesisConfig::for_local_testing);
        let account_keys = initial_accounts_config.generate_accounts(&mut self.rng).unwrap();

        let genesis = {
            let mut builder = GenesisStateBuilder::new();

            for validator in validator_set {
                builder = builder.add_validator(validator);
            }

            builder.build()
        };

        let validator_configs = validators
            .into_iter()
            .map(|validator| {
                let public_key: ValidatorPubKeyBytes = validator.key_pair.public().into();
                let db_path = self
                    .config_directory
                    .join(AUTHORITIES_DB_NAME)
                    .join(utils::encode_bytes_hex(&public_key));
                let network_address = validator.network_address;
                let consensus_address = validator.narwhal_consensus_address;
                let consensus_db_path = self
                    .config_directory
                    .join(CONSENSUS_DB_NAME)
                    .join(utils::encode_bytes_hex(&public_key));
                let consensus_config = ConsensusConfig {
                    consensus_address,
                    consensus_db_path,
                    narwhal_config: Default::default(),
                };

                NodeConfig {
                    key_pair: Arc::new(validator.key_pair),
                    db_path,
                    network_address,
                    metrics_address: utils::available_local_socket_address(),
                    admin_interface_port: utils::get_available_port(),
                    json_rpc_address: utils::available_local_socket_address(),
                    websocket_address: None,
                    consensus_config: Some(consensus_config),
                    enable_event_processing: false,
                    enable_gossip: true,
                    enable_reconfig: false,
                    genesis: crate::config::Genesis::new(genesis.clone()),
                }
            })
            .collect();

        NetworkConfig {
            validator_configs,
            genesis,
            account_keys,
        }
    }
}