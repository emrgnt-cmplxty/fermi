#[cfg(test)]
pub mod cluster_test_suite {

    // fermi
    use fermi_controller::bank::proto::create_create_asset_transaction;
    use fermi_core::multiaddr::to_socket_addr;
    use fermi_node::faucet_server::{FaucetService, FAUCET_PORT};
    use fermi_suite::test_utils::test_cluster::TestCluster;
    use fermi_types::{
        account::{AccountKeyPair, ValidatorKeyPair},
        asset::PRIMARY_ASSET_ID,
        block::BlockDigest,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        json_rpc::{BlockInfoReply, BlockReply},
        proto::{FaucetAirdropRequest, FaucetClient, FaucetServer, LatestBlockInfoRequest},
        transaction::{serialize_protobuf, ExecutedTransaction},
        utils,
    };
    // mysten
    use fastcrypto::{generate_production_keypair, Hash, DIGEST_LEN};
    use jsonrpsee::{http_client::HttpClientBuilder, rpc_params};
    use jsonrpsee_core::client::ClientT;
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::KeyPair;
    use narwhal_types::{Certificate, Header};
    // external
    use std::net::SocketAddr;
    use tokio::time::{sleep, Duration};
    use tonic::transport::Server;
    use tracing::info;

    // TESTS

    // TODO - I have marked all tests in this file as ignore because they cause file errors when run in parallel
    // We need to design a regression-style testing framework which runs this tests in nightly builds
    // We also need a scirpt that allows users to locally run these tests

    #[ignore]
    #[tokio::test]
    pub async fn test_spawn_cluster() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        cluster.send_transactions(0, 1, 10).await;
    }

    #[ignore]
    #[tokio::test]
    pub async fn test_balance_state() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;
        sleep(Duration::from_secs(2)).await;

        cluster
            .get_validator_spawner(1)
            .get_consensus_adapter()
            .unwrap()
            .update_batch_size(1);

        info!("Sending transactions");
        let (kp_sender, kp_receiver, _) = cluster.send_transactions(0, 1, 20).await;
        sleep(Duration::from_secs(5)).await;

        let genesis_state = cluster.get_validator_spawner(0).get_genesis_state();
        let sender_balance = genesis_state
            .controller_router()
            .bank_controller
            .lock()
            .unwrap()
            .get_balance(kp_sender.public(), PRIMARY_ASSET_ID)
            .unwrap();
        let receiver_balance = genesis_state
            .controller_router()
            .bank_controller
            .lock()
            .unwrap()
            .get_balance(kp_receiver.public(), PRIMARY_ASSET_ID)
            .unwrap();
        assert_eq!(sender_balance + receiver_balance, 2_500_000_000_000_000);
        assert!(receiver_balance > 0, "Receiver balance must be greater than 0");
    }

    #[ignore]
    #[tokio::test]
    pub async fn test_reconfigure_validator() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        cluster
            .get_validator_spawner(1)
            .get_consensus_adapter()
            .unwrap()
            .update_batch_size(1);

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

        let key = get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        spawner_0
            .get_tx_reconfigure_consensus()
            .as_ref()
            .unwrap()
            .send((key, new_committee))
            .await
            .unwrap();

        sleep(Duration::from_secs(1)).await;
    }

    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_cache_transactions() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        cluster
            .get_validator_spawner(1)
            .get_consensus_adapter()
            .unwrap()
            .update_batch_size(1);

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
            let transaction = signed_transaction.get_transaction().unwrap();
            assert!(validator_store.cache_contains_transaction(transaction));
        }

        let mut total = 0;
        let block_db = validator_store.critical_path_store.block_store.iter(None).await;
        let mut block_db_iter = block_db.iter();

        // TODO - more rigorously check exact match of transactions
        for next_block in block_db_iter.by_ref() {
            let block = next_block.1;
            for executed_transaction in &block.transactions {
                let transaction = executed_transaction.signed_transaction.get_transaction().unwrap();
                assert!(validator_store.cache_contains_transaction(transaction));
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
    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
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
    #[tokio::test]
    pub async fn test_validator_interface() {
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let spawner_1 = cluster.get_validator_spawner(1);
        let validator_state_1 = spawner_1.get_validator_state().unwrap();

        // Create txns
        let dummy_consensus_output = create_test_consensus_output();
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let transaction = create_create_asset_transaction(sender_kp.public(), recent_block_hash, 0);
        let signed_transaction = transaction.sign(&sender_kp).unwrap();

        // Preparing serialized buf for transactions
        let certificate = dummy_consensus_output.certificate;

        let initial_certificate = certificate.clone();

        let executed_transaction = ExecutedTransaction {
            signed_transaction: signed_transaction,
            events: Vec::new(),
            result: Ok(()),
        };
        // Write the block
        validator_state_1
            .validator_store
            .write_latest_block(initial_certificate, vec![executed_transaction])
            .await;

        // TODO make sure we have coverage for endpoints - https://github.com/fermiorg/fermi/issues/178
        let mut target_validator_client = cluster.get_validator_client(1);

        let latest_block_info_request = tonic::Request::new(LatestBlockInfoRequest {});
        let _latest_block_info_response = target_validator_client
            .get_latest_block_info(latest_block_info_request)
            .await;
    }

    #[ignore]
    #[tokio::test]
    pub async fn test_metrics() {
        let validator_count: usize = 4;
        const N_TRANSACTIONS: u64 = 1_000_000;

        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let receiver_adapter = cluster.get_validator_spawner(1).get_consensus_adapter().unwrap();
        receiver_adapter.update_batch_size(1);

        cluster.send_transactions(0, 1, 10).await;

        let metrics_0 = &cluster.get_validator_spawner(0).get_validator_state().unwrap().metrics;
        let metrics_1 = &cluster.get_validator_spawner(1).get_validator_state().unwrap().metrics;
        assert!(metrics_0.transactions_received.get() == 0);
        assert!(metrics_1.transactions_received.get() == 10);

        cluster
            .get_validator_spawner(1)
            .get_consensus_adapter()
            .unwrap()
            .update_batch_size(1);

        cluster.send_transactions_async(1, 0, N_TRANSACTIONS, None).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        assert!(metrics_0.get_average_tps() > 0.);
        assert!(metrics_0.get_average_latency_in_micros() > 0);
    }

    #[ignore]
    #[tokio::test]
    pub async fn test_spawn_faucet() {
        let temp_dir = tempfile::tempdir().unwrap();

        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let keypair: AccountKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let key_path = temp_dir.path().to_path_buf();
        let keystore_name = "validator-0.key";

        utils::write_keypair_to_file(&keypair, &key_path.join(&keystore_name)).unwrap();

        // Getting the address that is passed in
        let addr_str = format!("127.0.0.1:{}", FAUCET_PORT);

        // Parsing it into an address
        let addr = addr_str.parse::<SocketAddr>().unwrap();

        // Instantiating the faucet service
        let validator_addr = cluster.get_validator_spawner(0).get_grpc_address().clone();

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

    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn get_block() {
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let receiver_adapter = cluster.get_validator_spawner(1).get_consensus_adapter().unwrap();
        receiver_adapter.update_batch_size(1);

        cluster.send_transactions(0, 1, 1_000).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let spawner = cluster.get_validator_spawner(1);
        let json_rpc_addr = spawner.get_jsonrpc_address();

        let socket_addr = to_socket_addr(json_rpc_addr).unwrap();
        let url = format!("http://{}", socket_addr);

        let client = HttpClientBuilder::default().build(url).unwrap();

        let params = rpc_params![/* block_number */ 1];
        let _block_reply: Result<BlockReply, _> = client.request("tenex_getBlock", params).await;

        let params = rpc_params![/* block_number */ 1];
        let block_info: Result<BlockInfoReply, _> = client.request("tenex_getBlockInfo", params).await;

        let params = None;
        let latest_block_info: Result<BlockInfoReply, _> = client.request("tenex_getLatestBlockInfo", params).await;

        // check block numbers returned were greater than 1
        assert!(block_info.unwrap().block_number == 1);
        assert!(latest_block_info.unwrap().block_number > 1);
    }

    #[ignore]
    #[tokio::test]
    pub async fn test_json_rpc_submit_transaction() {
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        let receiver_adapter = cluster.get_validator_spawner(1).get_consensus_adapter().unwrap();
        receiver_adapter.update_batch_size(1);

        // Create txns
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let transaction = create_create_asset_transaction(sender_kp.public(), recent_block_hash, 0);
        let signed_transaction = transaction.sign(&sender_kp).unwrap();
        let signed_transaction_bytes = serialize_protobuf(&signed_transaction);
        let signed_transaction_hex = utils::encode_bytes_hex(&signed_transaction_bytes);

        println!("signed_transaction_hex={}", signed_transaction_hex);
        let spawner = cluster.get_validator_spawner(0);
        let json_rpc_addr = spawner.get_jsonrpc_address();

        let socket_addr = to_socket_addr(json_rpc_addr).unwrap();
        let url = format!("http://{}", socket_addr);

        let client = HttpClientBuilder::default().build(url).unwrap();

        let params = rpc_params![/* signed_transaction_bytes */ signed_transaction_hex];
        let _response: Result<String, _> = client.request("tenex_submitTransaction", params).await;
        println!("response={:?}", _response);
    }
}
