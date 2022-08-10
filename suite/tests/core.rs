#[cfg(test)]
pub mod suite_core_tests {
    use gdex_controller::master::MasterController;
    use gdex_core::{
        builder::genesis_state::GenesisStateBuilder,
        config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME, FULL_NODE_DB_PATH},
        genesis_ceremony::VALIDATOR_FUNDING_AMOUNT,
        metrics::start_prometheus_server,
        validator::{server::ValidatorService, state::ValidatorState},
    };
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorPubKeyBytes},
        crypto::KeypairTraits,
        node::ValidatorInfo,
        utils,
    };
    use std::sync::Arc;

    const VALIDATOR_SEED: [u8; 32] = [0; 32];

    #[tokio::test]
    pub async fn spawn_from_nodeconfig() {
        let temp_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_dir.path();
        let db_path = working_dir.join(FULL_NODE_DB_PATH);

        // Create a genesis config with just this validator
        let key_pair = generate_keypair_vec(VALIDATOR_SEED).pop().unwrap();
        let public_key = ValidatorPubKeyBytes::from(key_pair.public());
        let key_pair_pin = Arc::pin(key_pair);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: public_key.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let network_address = validator.network_address.clone();
        let consensus_address = validator.narwhal_consensus_address.clone();

        let builder = GenesisStateBuilder::new()
            .set_master_controller(MasterController::default())
            .add_validator(validator);

        let genesis_state = builder.build();
        let validator_state = Arc::new(ValidatorState::new(public_key, key_pair_pin, &genesis_state).await);
        let genesis = Genesis::new(genesis_state);

        // Create a node config with this validators informatin
        let consensus_db_path = working_dir.join(CONSENSUS_DB_NAME);
        let consensus_config = ConsensusConfig {
            consensus_address,
            consensus_db_path,
            narwhal_config: Default::default(),
        };

        // Re-initialize key_pair since we consumed our first copy in the genesis process
        let key_pair = generate_keypair_vec(VALIDATOR_SEED).pop().unwrap();

        let node_config = NodeConfig {
            key_pair: Arc::new(key_pair),
            db_path,
            network_address: network_address,
            metrics_address: utils::available_local_socket_address(),
            admin_interface_port: utils::get_available_port(),
            json_rpc_address: utils::available_local_socket_address(),
            websocket_address: Some(utils::available_local_socket_address()),
            consensus_config: Some(consensus_config),
            enable_event_processing: true,
            enable_gossip: true,
            enable_reconfig: false,
            genesis: genesis,
        };

        let prometheus_registry = start_prometheus_server(node_config.metrics_address);

        ValidatorService::new(&node_config, validator_state, &prometheus_registry)
            .await
            .unwrap();
    }
}
