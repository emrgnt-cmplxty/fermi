// TODO - how do we get set_testing_telemetry to work well with tests?
#[cfg(test)]
pub mod cluster_test_suite {

    // IMPORTS

    // gdex
    use gdex_core::{
        catchup::manager::{
            mock_catchup_manager::{MockCatchupManger, MockRelayServer},
            CatchupManager,
        },
        client::endpoint_from_multiaddr,
    };
    use gdex_node::faucet_server::{FaucetService, FAUCET_PORT};
    use gdex_suite::test_utils::test_cluster::TestCluster;
    use gdex_types::{
        account::AccountKeyPair,
        asset::PRIMARY_ASSET_ID,
        block::{Block, BlockDigest},
        crypto::{get_key_pair_from_rng, KeypairTraits, Signer},
        proto::{
            FaucetAirdropRequest, FaucetClient, FaucetServer, RelayerClient, RelayerGetBlockRequest,
            RelayerGetLatestBlockInfoRequest,
        },
        transaction::{create_asset_creation_transaction, SignedTransaction},
        utils,
    };

    // mysten
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::{generate_production_keypair, Hash, KeyPair, DIGEST_LEN};
    use narwhal_types::{Certificate, Header};

    // external
    use std::{net::SocketAddr, sync::Arc};
    use tokio::time::{sleep, Duration};
    use tonic::transport::Server;
    use tracing::info;

    // TESTS

    #[tokio::test]
    pub async fn test_spawn_cluster() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        cluster.send_transactions(0, 1, 10).await;
    }

    #[tokio::test]
    pub async fn test_balance_state() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;
        sleep(Duration::from_secs(2)).await;

        info!("Sending transactions");
        let (kp_sender, kp_receiver, _) = cluster.send_transactions(0, 1, 20).await;
        sleep(Duration::from_secs(5)).await;

        let genesis_state = cluster.get_validator_spawner(0).get_genesis_state();
        let sender_balance = genesis_state
            .master_controller()
            .bank_controller
            .lock()
            .unwrap()
            .get_balance(kp_sender.public(), PRIMARY_ASSET_ID)
            .unwrap();
        let receiver_balance = genesis_state
            .master_controller()
            .bank_controller
            .lock()
            .unwrap()
            .get_balance(kp_receiver.public(), PRIMARY_ASSET_ID)
            .unwrap();
        assert_eq!(sender_balance + receiver_balance, 2_500_000_000_000_000);
        assert!(receiver_balance > 0, "Receiver balance must be greater than 0");
    }

    #[tokio::test]
    pub async fn test_reconfigure_validator() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        cluster.send_transactions(0, 1, 10).await;

        sleep(Duration::from_secs(1)).await;

        info!("Reconfiguring validator");
        let spawner_0 = cluster.get_validator_spawner(0);
        let consensus_committee = spawner_0.get_genesis_state().narwhal_committee().load().clone();
        let new_committee: narwhal_config::Committee = narwhal_config::Committee::clone(&consensus_committee);
        let new_committee: narwhal_config::Committee = narwhal_config::Committee {
            authorities: new_committee.authorities,
            epoch: 1,
        };

        let key = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        spawner_0
            .get_tx_reconfigure_consensus()
            .as_ref()
            .unwrap()
            .send((key, new_committee))
            .await
            .unwrap();

        sleep(Duration::from_secs(1)).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_cache_transactions() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        let (_, _, signed_transactions) = cluster.send_transactions(0, 1, 10).await;

        info!("Sleep to allow all transactions to propagate");
        sleep(Duration::from_secs(5)).await;

        let spawner_1 = cluster.get_validator_spawner(1);
        let validator_store = &spawner_1
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        // check that every transaction entered the cache
        info!("Verify that all transactions entered cache");
        for signed_transaction in signed_transactions.clone() {
            assert!(validator_store.cache_contains_transaction(signed_transaction.get_transaction_payload()));
        }

        let mut total = 0;
        let block_db = validator_store.block_store.iter(None).await;
        let mut block_db_iter = block_db.iter();

        // TODO - more rigorously check exact match of transactions
        for next_block in block_db_iter.by_ref() {
            let block = next_block.1;
            for serialized_transaction in &block.transactions {
                let signed_transaction_db = SignedTransaction::deserialize(serialized_transaction.clone()).unwrap();
                assert!(validator_store.cache_contains_transaction(signed_transaction_db.get_transaction_payload()));
                total += 1;
            }
            assert!(validator_store.cache_contains_block_digest(&block.block_certificate.digest()));
        }
        assert!(
            total as u64 == signed_transactions.len() as u64,
            "total transactions in db does not match total submitted"
        );
    }

    /// TODO - This test currently fails because the stop function does not properly free the resources of the cluster
    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    pub async fn test_stop_start_node() {
        let validator_count: usize = 4;
        let target_node = validator_count - 1;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Stoping target_node={target_node}");
        cluster.stop(target_node).await;
        info!("Sleeping 10s to give more than enough time for shutdown");
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        info!("Starting target_node={target_node}");
        cluster.start(target_node).await;
        info!("Sleeping 10s to give time for node to restart and have a potential error");
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }

    pub fn create_test_consensus_output() -> ConsensusOutput {
        let dummy_header = Header::default();
        let dummy_certificate = Certificate {
            header: dummy_header,
            votes: Vec::new(),
        };
        ConsensusOutput {
            certificate: dummy_certificate,
            consensus_index: 1,
        }
    }

    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_catchup_new_node_mock() {
        // utils::set_testing_telemetry("gdex_core=info, gdex_suite=info");
        // submit more transactions than we can possibly process
        const N_TRANSACTIONS: u64 = 1_000_000;
        info!("Creating test cluster");
        let validator_count: usize = 5;
        let target_node = validator_count - 1;

        info!("Launching nodes 1 - {}", target_node);
        let mut cluster = TestCluster::spawn(validator_count, Some(target_node)).await;

        info!("Begin Sending {N_TRANSACTIONS} transactions");
        cluster.send_transactions_async(0, 1, N_TRANSACTIONS, None).await;

        info!("Sleeping 5s to allow network to advance circulation");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Booting up node {}", target_node + 1);
        cluster.start(target_node).await;

        let validator_store_node_1 = &cluster
            .get_validator_spawner(0)
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        let restarted_validator_state = cluster
            .get_validator_spawner(target_node)
            .get_validator_state()
            .unwrap();

        // Verify that blocks do not match before running catchup
        let latest_block_store_node_0 = validator_store_node_1
            .last_block_info_store
            .read(0)
            .await
            .expect("Error fetching from the last block store")
            .expect("Latest block info for node 0 was unexpectedly empty");

        let latest_block_store_target = restarted_validator_state
            .validator_store
            .last_block_info_store
            .read(0)
            .await
            .expect("Error fetching from the last block store")
            // allow unwrap to default for this special case
            .unwrap_or_default();

        assert!(latest_block_store_node_0.block_number != latest_block_store_target.block_number);

        let mock_server = MockRelayServer::new(validator_store_node_1);
        let mut mock_catchup_manager = MockCatchupManger::new(10);
        mock_catchup_manager
            .catchup_narwhal_mediated(&mock_server, &restarted_validator_state)
            .await
            .unwrap();

        // drop the cluster to stop forward progress of consensus
        drop(cluster);

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Verify that blocks do match after running catchup
        let latest_block_store_node_1 = validator_store_node_1
            .last_block_info_store
            .read(0)
            .await
            .expect("Error fetching from the last block store")
            .expect("Latest block info for node 0 was unexpectedly empty");

        let latest_block_store_target = restarted_validator_state
            .validator_store
            .last_block_info_store
            .read(0)
            .await
            .expect("Error fetching from the last block store")
            .expect("Latest block info for target node was unexpectedly empty");

        // verify that blocks do match after running catchup
        assert!(
            latest_block_store_node_1.block_number == latest_block_store_target.block_number,
            "Failure, catchup node block number = {}, target node block number = {}",
            latest_block_store_node_1.block_number,
            latest_block_store_target.block_number
        );
        info!("Success");
    }

    // TODO - investigate buiding helper functions for relay client
    // TODO - can we remove the ignore decorator without CI failures?
    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_catchup_new_node() {
        // utils::set_testing_telemetry("gdex_core=info, gdex_suite=info");
        // submit more transactions than we can possibly process
        const N_TRANSACTIONS: u64 = 1_000_000;
        info!("Creating test cluster");
        let validator_count: usize = 5;
        let target_node = validator_count - 1;

        info!("Launching nodes 1 - {}", target_node);
        let mut cluster = TestCluster::spawn(validator_count, Some(target_node)).await;

        info!("Begin Sending {N_TRANSACTIONS} transactions");
        cluster.send_transactions_async(0, 1, N_TRANSACTIONS, None).await;

        info!("Sleeping 5s to allow network to advance circulation");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Booting up node {}", target_node + 1);
        cluster.start(target_node).await;

        // fetch the relay spawner prior to the node we are catching up with
        let relayer_prev_target = cluster.spawn_single_relayer(target_node - 1).await;
        let relayer_endpoint = endpoint_from_multiaddr(&relayer_prev_target.get_relayer_address()).unwrap();
        let mut non_target_relayer = RelayerClient::connect(relayer_endpoint.endpoint().clone())
            .await
            .unwrap();

        let spawner = cluster.get_validator_spawner(target_node);
        let validator_state = spawner.get_validator_state().unwrap();

        let relayer_target = cluster.spawn_single_relayer(target_node).await;
        let relayer_endpoint = endpoint_from_multiaddr(&relayer_target.get_relayer_address()).unwrap();
        let mut target_relayer = RelayerClient::connect(relayer_endpoint.endpoint().clone())
            .await
            .unwrap();

        // do catch-up

        let mut catchup_manager = CatchupManager::new(non_target_relayer.clone(), Arc::clone(&validator_state));
        catchup_manager.catchup_narwhal_mediated().await.unwrap();

        // drop the cluster to stop forward progress of consensus
        drop(cluster);

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // check post-catchup states
        let latest_block_info_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {});
        let latest_block_info_response = non_target_relayer
            .get_latest_block_info(latest_block_info_request)
            .await;
        let block_info_non_target = latest_block_info_response.unwrap().into_inner().block_info.unwrap();

        let latest_block_info_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {});
        let latest_block_info_response = target_relayer.get_latest_block_info(latest_block_info_request).await;
        let block_info_target = latest_block_info_response.unwrap().into_inner().block_info.unwrap();

        // check that we are fully caught up to the target node
        assert!(block_info_non_target.block_number == block_info_target.block_number);
    }

    #[tokio::test]
    pub async fn test_spawn_relayer() {
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let spawner_1 = cluster.get_validator_spawner(1);
        let validator_state_1 = spawner_1.get_validator_state().unwrap();

        // Create txns
        let dummy_consensus_output = create_test_consensus_output();
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash, 0);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        // Preparing serialized buf for transactions
        let mut serialized_txns_buf: Vec<Vec<u8>> = Vec::new();
        let serialized_txn = signed_create_asset_txn.serialize().unwrap();
        serialized_txns_buf.push(serialized_txn);
        let certificate = dummy_consensus_output.certificate;

        let initial_certificate = certificate.clone();
        let initial_serialized_txns_buf = serialized_txns_buf.clone();

        // Write the block
        validator_state_1
            .validator_store
            .write_latest_block(initial_certificate, initial_serialized_txns_buf)
            .await;

        // TODO clean

        let relayer_1 = cluster.spawn_single_relayer(1).await;
        let target_endpoint = endpoint_from_multiaddr(&relayer_1.get_relayer_address()).unwrap();
        let endpoint = target_endpoint.endpoint();
        let mut client = RelayerClient::connect(endpoint.clone()).await.unwrap();

        let specific_block_request = tonic::Request::new(RelayerGetBlockRequest { block_number: 0 });
        let latest_block_info_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {});

        // Act
        let specific_block_response = client.get_block(specific_block_request).await;

        let _latest_block_info_response = client.get_latest_block_info(latest_block_info_request).await;

        let block_bytes_returned = specific_block_response.unwrap().into_inner().block.unwrap().block;

        // Assert
        let deserialized_block: Block = bincode::deserialize(&block_bytes_returned).unwrap();

        let final_certificate = certificate.clone();
        let final_serialized_txns_buf = serialized_txns_buf.clone();
        let block_to_check_against = Block {
            block_certificate: final_certificate,
            transactions: final_serialized_txns_buf,
        };

        assert!(block_to_check_against.block_certificate == deserialized_block.block_certificate);
        assert!(block_to_check_against.transactions == deserialized_block.transactions);
        // assert!(latest_block_info_response.unwrap().into_inner().successful)
    }

    #[tokio::test]
    pub async fn test_metrics() {
        let validator_count: usize = 4;
        const N_TRANSACTIONS: u64 = 1_000_000;

        let mut cluster = TestCluster::spawn(validator_count, None).await;
        cluster.send_transactions(0, 1, 10).await;

        let metrics_0 = &cluster.get_validator_spawner(0).get_validator_state().unwrap().metrics;
        let metrics_1 = &cluster.get_validator_spawner(1).get_validator_state().unwrap().metrics;
        assert!(metrics_0.num_transactions_rec.load(std::sync::atomic::Ordering::SeqCst) == 0);
        assert!(metrics_1.num_transactions_rec.load(std::sync::atomic::Ordering::SeqCst) == 10);

        cluster.send_transactions_async(1, 0, N_TRANSACTIONS, None).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let init_time = metrics_0
            .latest_system_epoch_time_in_ms
            .load(std::sync::atomic::Ordering::SeqCst);
        let init_transactions = metrics_0
            .num_transactions_consensus
            .load(std::sync::atomic::Ordering::SeqCst);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let time_delta = metrics_0
            .latest_system_epoch_time_in_ms
            .load(std::sync::atomic::Ordering::SeqCst)
            - init_time;
        let transactions_delta = metrics_0
            .num_transactions_consensus
            .load(std::sync::atomic::Ordering::SeqCst)
            - init_transactions;

        // assert tps was greater than 0
        assert!(transactions_delta as f64 / (time_delta as f64 / 1000.) > 0.);
    }

    pub async fn test_spawn_faucet() {
        let temp_dir = tempfile::tempdir().unwrap();

        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let keypair: AccountKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let key_path = temp_dir.path().to_path_buf();
        let keystore_name = "validator-0.key";

        utils::write_keypair_to_file(&keypair, &key_path.join(&keystore_name)).unwrap();

        // Getting the address that is passed in
        let addr_str = format!("127.0.0.1:{}", FAUCET_PORT);

        // Parsing it into an address
        let addr = addr_str.parse::<SocketAddr>().unwrap();

        // Instantiating the faucet service
        let validator_addr = cluster.get_validator_spawner(0).get_validator_address().clone();

        let faucet_service = FaucetService {
            validator_index: 0,
            key_path,
            validator_addr,
        };
        tokio::spawn(async move {
            Server::builder()
                .add_service(FaucetServer::new(faucet_service))
                .serve(addr)
                .await
                .unwrap();
        });

        sleep(Duration::from_secs(1)).await;

        let receiver_kp = generate_production_keypair::<KeyPair>();

        let addr_str = format!("http://127.0.0.1:{}", FAUCET_PORT);
        let mut client = FaucetClient::connect(addr_str.to_string()).await.unwrap();

        let request = tonic::Request::new(FaucetAirdropRequest {
            // airdrop_to: hex::encode(receiver_kp.public().to_string()),
            airdrop_to: utils::encode_bytes_hex(receiver_kp.public()),
            amount: 100,
        });

        let response = client.airdrop(request).await.unwrap().into_inner();

        assert!(response.successful);
    }
}
