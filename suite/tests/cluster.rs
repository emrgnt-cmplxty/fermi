#[cfg(test)]
pub mod cluster_test_suite {

    // IMPORTS
    
    // external
    use std::{
        io
    };
    use tracing::info;
    use tracing_subscriber::FmtSubscriber;
    
    // mysten
    
    // gdex
    use gdex_core::{
        client
    };
    use gdex_types::{
        account::{
            account_test_functions::generate_keypair_vec,
            ValidatorKeyPair
        },
        proto::{
            TransactionProto,
            TransactionsClient
        },
        crypto::get_key_pair_from_rng,
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };
    
    // local
    use gdex_suite::{
        test_utils::{
            test_cluster::TestCluster
        }
    };

    // TESTS

    #[tokio::test]
    pub async fn test_spawn_cluster() {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter("gdex_core=trace, gdex_suite=debug")
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let cluster = TestCluster::new(validator_count).await;
        
        let spawner_0 = cluster.get_validator_spawner(0);
        
        info!("Sending transactions");
        let working_dir = cluster.get_working_dir();
        let key_file = working_dir.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        info!("Connecting network client to address={:?}", address);

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        info!("Sending transactions");
        let mut i = 0;
        while i < 1_000 {
            let transaction_proto = TransactionProto {
                transaction: signed_transaction.serialize().unwrap().into(),
            };
            let _resp1 = client
                .submit_transaction(transaction_proto)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
            i += 1;
        }
    }

    #[tokio::test]
    pub async fn test_spawn_cluster() {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter("gdex_core=trace, gdex_suite=debug")
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let cluster = TestCluster::new(validator_count).await;
        
        let spawner_0 = cluster.get_validator_spawner(0);
        
        info!("Sending transactions");
        let working_dir = cluster.get_working_dir();
        let key_file = working_dir.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        info!("Connecting network client to address={:?}", address);

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        info!("Sending transactions");
        let mut i = 0;
        while i < 10 {
            let transaction_proto = TransactionProto {
                transaction: signed_transaction.serialize().unwrap().into(),
            };
            let _resp1 = client
                .submit_transaction(transaction_proto)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
            i += 1;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        info!("Reconfiguring validator");

        let consensus_committee = spawner_0.get_genesis_state().narwhal_committee().load().clone();
        let new_committee: narwhal_config::Committee = narwhal_config::Committee::clone(&consensus_committee);
        let new_committee: narwhal_config::Committee = narwhal_config::Committee {
            authorities: new_committee.authorities,
            epoch: 1,
        };

        let key = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        spawner_0.tx_reconfigure_consensus.unwrap().send((key, new_committee)).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    #[tokio::test]
    pub async fn test_cache_transactions() {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter("gdex_core=trace, gdex_suite=debug")
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    
        info!("Creating test cluster");
        let validator_count: usize = 4;
        let cluster = TestCluster::new(validator_count).await;
        
        let spawner_0 = cluster.get_validator_spawner(0);
        let spawner_1 = cluster.get_validator_spawner(0);
        
        info!("Sending transactions");
        let working_dir = cluster.get_working_dir();
        let key_file = working_dir.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        info!("Connecting network client to address={:?}", address);

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        info!("Sending transactions");
        let mut i = 0;
        let mut signed_transactions = Vec::new();
        while i < 10 {
            let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver, i);
            signed_transactions.push(signed_transaction.clone());
            let transaction_proto = TransactionProto {
                transaction: signed_transaction.serialize().unwrap().into(),
            };
            let _resp1 = client
                .submit_transaction(transaction_proto)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
            i += 1;
        }
        
        info!("Sleep to allow all transactions to propagate");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let validator_store = &spawner_1
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        // check that every transaction entered the cache
        info!("Verify that all transactions entered cache")
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
            assert!(validator_store.contains_block_digest(&block.block_digest));
        }
        assert!(
            total as u64 == n_transactions_to_submit,
            "total transactions in db does not match total submitted"
        );
        
    }
}
