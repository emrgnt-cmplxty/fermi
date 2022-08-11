#[cfg(test)]
pub mod suite_core_tests {
    use gdex_controller::master::MasterController;
    use gdex_core::{
        client::{ClientAPI, NetworkValidatorClient},
        config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME, FULL_NODE_DB_PATH},
        genesis_ceremony::VALIDATOR_FUNDING_AMOUNT,
        metrics::start_prometheus_server,
        validator::{
            genesis_state::ValidatorGenesisState, server::ValidatorServer, server::ValidatorService,
            state::ValidatorState,
        },
    };
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair, ValidatorPubKeyBytes},
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };
    use narwhal_config::Parameters as ConsensusParameters;
    use std::path::Path;
    use std::{io, sync::Arc, time};
    use tracing_subscriber::FmtSubscriber;

    // Create a genesis config with a single validator seeded by VALIDATOR_SEED
    async fn get_genesis_state(dir: &Path, number_of_validators: usize) -> ValidatorGenesisState {
        let validators = (0..number_of_validators)
            .map(|i| {
                let keypair: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
                let info = ValidatorInfo {
                    name: format!("validator-{i}"),
                    public_key: ValidatorPubKeyBytes::from(keypair.public()),
                    stake: VALIDATOR_FUNDING_AMOUNT,
                    delegation: 0,
                    network_address: utils::new_network_address(),
                    narwhal_primary_to_primary: utils::new_network_address(),
                    narwhal_worker_to_primary: utils::new_network_address(),
                    narwhal_primary_to_worker: utils::new_network_address(),
                    narwhal_worker_to_worker: utils::new_network_address(),
                    narwhal_consensus_address: utils::new_network_address(),
                };
                let key_file = dir.join(format!("{}.key", info.name));
                utils::write_keypair_to_file(&keypair, &key_file).unwrap();
                info
            })
            .collect::<Vec<_>>();

        ValidatorGenesisState::new(MasterController::default(), validators)
    }

    // Start a new validator service given a genesis state and the index of the validator we are using for the local test
    // note that the validator is generated by the seed VALIDATOR_SEED
    async fn start_validator_service(
        dir: &Path,
        batch_size: usize,
        max_batch_delay: time::Duration,
        genesis_state: ValidatorGenesisState,
        validator_index: usize,
    ) -> (Arc<ValidatorState>, ValidatorService) {
        // create config directory
        let temp_dir = tempfile::tempdir().unwrap();
        let db_dir = temp_dir.path();
        let db_path = db_dir.join(FULL_NODE_DB_PATH);

        let validators = genesis_state.validator_set();
        assert!(
            validators.len() > validator_index,
            "Something went wrong in generating the validator set."
        );

        let validator = validators[validator_index].clone();
        println!("validator={:?}", validator);
        let network_address = validator.network_address.clone();
        let consensus_address = validator.narwhal_consensus_address.clone();
        let pubilc_key = validator.public_key();

        // TODO - can we avoid consuming the private key twice in the network setup?
        // Note, this awkwardness is due to my inferred understanding of Arc pin.
        let key_file = dir.join(format!("{}.key", validator.name));

        let key_pair_pin = Arc::pin(utils::read_keypair_from_file(&key_file).unwrap());
        let key_pair_arc = Arc::new(utils::read_keypair_from_file(&key_file).unwrap());

        let validator_state = Arc::new(ValidatorState::new(pubilc_key, key_pair_pin, &genesis_state).await);

        // Create a node config with this validators information
        let consensus_db_path = db_dir.join(CONSENSUS_DB_NAME);
        let narwhal_config: ConsensusParameters = ConsensusParameters {
            batch_size,
            max_batch_delay,
            ..Default::default()
        };

        let consensus_config = ConsensusConfig {
            consensus_address,
            consensus_db_path,
            narwhal_config,
        };

        let node_config = NodeConfig {
            key_pair: key_pair_arc,
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
            genesis: Genesis::new(genesis_state),
        };

        // spawn the validator service, e.g. Narwhal consensus
        let prometheus_registry = start_prometheus_server(node_config.metrics_address);
        let spawned_service = ValidatorService::new(&node_config, Arc::clone(&validator_state), &prometheus_registry)
            .await
            .unwrap();
        (validator_state, spawned_service)
    }

    const NUMBER_OF_TEST_VALIDATORS: usize = 4;
    #[tokio::test] #[ignore] // it fails in remote view
    pub async fn four_node_network() {
        // let subscriber = FmtSubscriber::builder()
        //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        //     // will be written to stdout.
        //     .with_env_filter("gdex_core=debug, gdex_suite=debug")
        //     // .with_max_level(Level::DEBUG)
        //     // completes the builder.
        //     .finish();
        // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        let temp_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_dir.path();

        let genesis_state = get_genesis_state(working_dir, NUMBER_OF_TEST_VALIDATORS).await;
        let validator_index = 0;

        let batch_size = 100;
        let max_delay = time::Duration::from_millis(1_000_000);

        let (validator_state, validator_service) = start_validator_service(
            working_dir,
            batch_size,
            max_delay,
            genesis_state.clone(),
            validator_index,
        )
        .await;

        start_validator_service(working_dir, batch_size, max_delay, genesis_state.clone(), 1).await;
        start_validator_service(working_dir, batch_size, max_delay, genesis_state.clone(), 2).await;
        start_validator_service(working_dir, batch_size, max_delay, genesis_state, 3).await;

        let new_addr = utils::new_network_address();
        let consensus_address = validator_service
            .consensus_adapter
            .lock()
            .await
            .consensus_address
            .to_owned();
        let validator_server = ValidatorServer::new(
            new_addr.clone(),
            Arc::clone(&validator_state),
            consensus_address,
            Some(batch_size),
            Some(max_delay),
        );
        let validator_handle = validator_server.spawn().await.unwrap();

        let key_file = working_dir.join(format!("validator-{}.key", validator_index));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let client = NetworkValidatorClient::connect_lazy(&validator_handle.address()).unwrap();
        let mut i = 0;
        while i < 1_000 {
            let _resp1 = client
                .handle_transaction(signed_transaction.clone())
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
            i += 1;
        }
    }
}
