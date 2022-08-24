#[cfg(test)]
pub mod cluster_test_suite {

    // IMPORTS

    // local
    use gdex_suite::test_utils::test_cluster::TestCluster;
    use gdex_core::{
        catchup::mock_catchup_manager::{MockCatchupManger, MockRelayServer},
    };
    use gdex_types::{
        asset::PRIMARY_ASSET_ID,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        transaction::SignedTransaction,
    };

    // external
    use tracing::info;
    use tokio::time::{sleep, Duration};


    // TESTS

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_spawn_cluster() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        cluster.send_transactions(0, 1, 1_000, None).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_balance_state() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        let (kp_sender, kp_receiver, _) = cluster.send_transactions(0, 1, 20, Some(1_000_000)).await;

        sleep(Duration::from_secs(3)).await;

        let genesis_state = cluster.get_validator_spawner(0).get_genesis_state();
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

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_reconfigure_validator() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        cluster.send_transactions(0, 1, 10, None).await;

        sleep(Duration::from_secs(1)).await;

        info!("Reconfiguring validator");
        let spawner = cluster.get_validator_spawner(0);
        let consensus_committee = spawner.get_genesis_state().narwhal_committee().load().clone();
        let new_committee: narwhal_config::Committee = narwhal_config::Committee::clone(&consensus_committee);
        let new_committee: narwhal_config::Committee = narwhal_config::Committee {
            authorities: new_committee.authorities,
            epoch: 1,
        };

        let key = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        spawner
            .get_tx_reconfigure_consensus()
            .as_ref()
            .unwrap()
            .send((key, new_committee))
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_cache_transactions() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::spawn(validator_count, None).await;

        info!("Sending transactions");
        let (_, _, signed_transactions) = cluster.send_transactions(0, 1, 10, None).await;

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
            assert!(validator_store.contains_transaction(&signed_transaction.get_transaction_payload()));
        }

        let mut total = 0;
        let block_db = validator_store.block_store.iter(None).await;
        let mut block_db_iter = block_db.iter();

        // TODO - more rigorously check exact match of transactions
        while let Some(next_block) = block_db_iter.next() {
            let block = next_block.1;
            for serialized_transaction in &block.transactions {
                let signed_transaction_db = SignedTransaction::deserialize(serialized_transaction.clone()).unwrap();
                assert!(validator_store.contains_transaction(&signed_transaction_db.get_transaction_payload()));
                total += 1;
            }
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

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_catchup_new_node_mock() {
        telemetry_subscribers::init_for_testing();

        // submit more transactions than we can possibly process
        const N_TRANSACTIONS: u64 = 1_000_000;

        info!("Creating test cluster");
        let validator_count: usize = 6;
        let target_node = validator_count - 1;
        info!("Launching nodes 1 - {}", target_node);
        let mut cluster = TestCluster::spawn(validator_count, Some(target_node)).await;

        info!("Begin Sending {N_TRANSACTIONS} transactions");
        cluster.send_transactions_async(0, 1, N_TRANSACTIONS, None).await;

        info!("Sleeping 5s to allow network to advance circulation");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        info!("Booting up node {}", target_node + 1);
        cluster.start(target_node).await;

        info!("Sleeping 250ms to allow boot");
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

        let validator_store = &cluster
            .get_validator_spawner(1)
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        let restarted_validator_state = cluster
            .get_validator_spawner(target_node)
            .get_validator_state()
            .unwrap();

        let mock_server = MockRelayServer::new(validator_store);
        let mut mock_catchup_manager = MockCatchupManger::new(10);
        mock_catchup_manager
            .catchup_narwhal_mediated(&mock_server, &restarted_validator_state)
            .await
            .unwrap();

        drop(cluster);
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }
}
