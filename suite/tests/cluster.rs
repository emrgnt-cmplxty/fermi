#[cfg(test)]
pub mod cluster_test_suite {

    // IMPORTS

    // external
    use std::io;
    use tracing::info;
    //use tracing_subscriber::FmtSubscriber;
    use tokio::time::{sleep, Duration};

    // mysten

    // gdex
    use gdex_core::{catchup::{CatchupManager, mock_catchup_manager::MockRelayServer}, client, validator::spawner::ValidatorSpawner};
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair},
        asset::PRIMARY_ASSET_ID,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        proto::{TransactionProto, TransactionsClient},
        transaction::{transaction_test_functions::generate_signed_test_transaction, SignedTransaction},
        utils,
    };

    // local
    use gdex_suite::test_utils::test_cluster::TestCluster;

    // TESTS

    #[tokio::test]
    pub async fn test_spawn_cluster() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::new(validator_count).await;
        
        info!("Sending transactions");
        cluster.send_transactions(0, 1, 1_000, None).await;

    }

    #[tokio::test]
    pub async fn test_balance_state() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::new(validator_count).await;

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

    #[tokio::test]
    pub async fn test_reconfigure_validator() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::new(validator_count).await;

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

    #[tokio::test]
    pub async fn test_cache_transactions() {
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let mut cluster = TestCluster::new(validator_count).await;

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

    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_node_catchup() {
        let validator_count: usize = 4;
        let mut cluster = TestCluster::new(validator_count).await;
        cluster.stop(validator_count - 1).await;
        // create a thread where 1_000_000 transactions are sent
        let _handle = cluster.send_transactions_async(0, 1, 1_000_000, None).await;

        // tokio::spawn(async move {
        //     let (_, _, signed_transactions) = cluster.send_transactions(0, 1, 10, None).await;
        // });
        println!("sleeping now..");
        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        println!("continuing now..");
        cluster.start(validator_count - 1).await;
        let spawner_0 = cluster.get_validator_spawner(0);
        // let spawner_restarted = cluster.get_validator_spawner(validator_count - 1);

        let validator_store = &spawner_0
        .get_validator_state()
        .as_ref()
        .unwrap()
        .clone()
        .validator_store;

        let restarted_validator_state = cluster.get_validator_spawner(validator_count - 1).get_validator_state().unwrap();


        let mut mock_catchup_manager = CatchupManager::new();
        let mock_server = MockRelayServer::new(validator_store);
        mock_catchup_manager
            .catchup_to_latest_block(&mock_server, &restarted_validator_state)
            .await
            .unwrap();

        // for i in 1..10000 {
        //     println!("spawner_0 latest block = ", spawner_0.get_validator_state().validator_store.latest_block_id());
        // }


    }
}
