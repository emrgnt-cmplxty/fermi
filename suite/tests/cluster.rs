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
    pub async fn spawn_cluster() {
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
}
