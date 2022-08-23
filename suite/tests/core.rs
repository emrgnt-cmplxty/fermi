#[cfg(test)]
pub mod suite_core_tests {
    use gdex_controller::{bank::CREATED_ASSET_BALANCE, master::MasterController};
    use gdex_core::{
        client,
        config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME, GDEX_DB_NAME},
        genesis_ceremony::{VALIDATOR_BALANCE, VALIDATOR_FUNDING_AMOUNT},
        metrics::start_prometheus_server,
        validator::{
            genesis_state::ValidatorGenesisState, server::ValidatorServer, server::ValidatorService,
            state::ValidatorState,
        },
    };
    use gdex_types::{
        account::{
            account_test_functions::generate_keypair_vec, ValidatorKeyPair, ValidatorPubKey, ValidatorPubKeyBytes,
        },
        asset::PRIMARY_ASSET_ID,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        proto::{TransactionProto, TransactionsClient},
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };
    use narwhal_config::Parameters as ConsensusParameters;
    use std::path::Path;
    use std::{io, sync::Arc, time};
    use tokio::task::JoinHandle;
    use tokio::time::{sleep, Duration};

    // Create a genesis config with a single validator seeded by VALIDATOR_SEED
    async fn get_genesis_state(dir: &Path, number_of_validators: u64) -> ValidatorGenesisState {
        let validators = (0..number_of_validators)
            .map(|i| {
                let keypair: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
                let info = ValidatorInfo {
                    name: format!("validator-{i}"),
                    public_key: ValidatorPubKeyBytes::from(keypair.public()),
                    stake: VALIDATOR_FUNDING_AMOUNT,
                    balance: VALIDATOR_BALANCE,
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

        let master_controller = MasterController::default();
        // create asset + fund other validators
        let validator_creator_pubkey = ValidatorPubKey::try_from(validators[0].public_key).unwrap();
        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(&validator_creator_pubkey)
            .unwrap();
        // get transfer amount
        let transfer_amount: u64 = CREATED_ASSET_BALANCE / number_of_validators;
        for i in 1..number_of_validators {
            let validator_pubkey = ValidatorPubKey::try_from(validators[i as usize].public_key).unwrap();
            master_controller
                .bank_controller
                .lock()
                .unwrap()
                .transfer(
                    &validator_creator_pubkey,
                    &validator_pubkey,
                    PRIMARY_ASSET_ID,
                    transfer_amount,
                )
                .unwrap();
        }

        ValidatorGenesisState::new(master_controller, validators)
    }

    // Start a new validator service given a genesis state and the index of the validator we are using for the local test
    // note that the validator is generated by the seed VALIDATOR_SEED
    async fn spawn_validator_service(
        dir: &Path,
        batch_size: usize,
        max_batch_delay: time::Duration,
        genesis_state: ValidatorGenesisState,
        validator_index: usize,
    ) -> (Arc<ValidatorState>, Vec<JoinHandle<()>>) {
        // create config directory
        let temp_dir = tempfile::tempdir().unwrap();
        let db_dir = temp_dir.path();

        let validators = genesis_state.validator_set();
        assert!(
            validators.len() > validator_index,
            "Something went wrong in generating the validator set."
        );

        let validator = validators[validator_index].clone();
        let network_address = validator.network_address.clone();
        let consensus_address = validator.narwhal_consensus_address.clone();
        let public_key = validator.public_key();

        // TODO - can we avoid consuming the private key twice in the network setup?
        // Note, this awkwardness is due to my inferred understanding of Arc pin.
        let key_file = dir.join(format!("{}.key", validator.name));

        let key_pair_pin = Arc::pin(utils::read_keypair_from_file(&key_file).unwrap());
        let key_pair_arc = Arc::new(utils::read_keypair_from_file(&key_file).unwrap());
        let gdex_db_path = db_dir.join(GDEX_DB_NAME);
        let validator_state = Arc::new(ValidatorState::new(
            public_key,
            key_pair_pin,
            &genesis_state,
            &gdex_db_path,
        ));

        // Create a node config with this validators information
        let consensus_db_path = db_dir.join(CONSENSUS_DB_NAME);
        let narwhal_config: ConsensusParameters = ConsensusParameters {
            batch_size,
            max_batch_delay,
            ..Default::default()
        };

        let consensus_config = ConsensusConfig {
            consensus_address,
            consensus_db_path: consensus_db_path.clone(),
            narwhal_config,
        };

        let node_config = NodeConfig {
            key_pair: key_pair_arc,
            consensus_db_path,
            gdex_db_path,
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
        let (_tx_reconfigure_consensus, rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);
        let narwhal_handle = ValidatorService::spawn_narwhal(
            &node_config,
            Arc::clone(&validator_state),
            &prometheus_registry,
            rx_reconfigure_consensus,
        )
        .await
        .unwrap();
        (validator_state, narwhal_handle)
    }

    const NUMBER_OF_TEST_VALIDATORS: u64 = 4;
    #[tokio::test]
    #[ignore] // it fails in remote view
    pub async fn four_node_network() {
        let temp_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_dir.path();

        let genesis_state = get_genesis_state(working_dir, NUMBER_OF_TEST_VALIDATORS).await;

        let batch_size = 100;
        let max_delay = time::Duration::from_millis(1_000_000);

        let primary_validator_index = 0;
        let validator = genesis_state.validator_set()[primary_validator_index].clone().clone();

        let (validator_state, _narwhal_handle) = spawn_validator_service(
            working_dir,
            batch_size,
            max_delay,
            genesis_state.clone(),
            primary_validator_index,
        )
        .await;

        spawn_validator_service(working_dir, batch_size, max_delay, genesis_state.clone(), 1).await;
        spawn_validator_service(working_dir, batch_size, max_delay, genesis_state.clone(), 2).await;
        spawn_validator_service(working_dir, batch_size, max_delay, genesis_state.clone(), 3).await;

        let new_addr = utils::new_network_address();
        let consensus_address = validator.narwhal_consensus_address.clone();

        let (tx_reconfigure_consensus, _rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);
        let validator_server = ValidatorServer::new(
            new_addr.clone(),
            Arc::clone(&validator_state),
            consensus_address,
            tx_reconfigure_consensus,
        );
        let validator_handle = validator_server.spawn().await.unwrap();

        let key_file = working_dir.join(format!("validator-{}.key", primary_validator_index));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver, 1_000_000);

        let mut client = TransactionsClient::new(
            client::connect_lazy(&validator_handle.address()).expect("Failed to connect to consensus"),
        );
        for _ in 0..20 {
            let transaction_proto = TransactionProto {
                transaction: signed_transaction.serialize().unwrap().into(),
            };
            let _resp1 = client
                .submit_transaction(transaction_proto)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
        }
        sleep(Duration::from_millis(3250)).await;
        let sender_balance = genesis_state
            .clone()
            .master_controller()
            .bank_controller
            .lock()
            .unwrap()
            .get_balance(&kp_sender.public(), PRIMARY_ASSET_ID)
            .unwrap();
        let receiver_balance = genesis_state
            .clone()
            .master_controller()
            .bank_controller
            .lock()
            .unwrap()
            .get_balance(&kp_receiver.public(), PRIMARY_ASSET_ID)
            .unwrap();
        assert_eq!(sender_balance + receiver_balance, 2_500_000_000_000_000);
        assert!(receiver_balance > 0, "Receiver balance must be greater than 0");
    }
}
