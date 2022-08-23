//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// TODO -
// Add networking logic to the validator state
// Add a way to get the latest block info from the validator state
// Add a way to get latest committee from validators
// Add catch-up logic to spawner, have it take into account the latest data in the database
// Add logic around when to catch-up and when not to

/// This module provides functionality to fetch the latest block given a mock relay sever
/// moreover, the module then iterates until the latest block number of the passed validator  
/// state matches the mock block number of the newly generated validator store
///
/// the intention of this function is to use as a mock for testing
/// this functionality will be used in the future if Narwhal implements garbage collection
use crate::validator::state::ValidatorState;
use gdex_types::block::BlockNumber;
use gdex_types::error::GDEXError;
use std::sync::Arc;

// #[cfg(test)]
pub mod mock_catchup_manager {
    use crate::validator::state::{ValidatorState, ValidatorStore};
    use gdex_types::{
        block::{Block, BlockInfo, BlockNumber},
        error::GDEXError,
    };
    use std::sync::Arc;

    const MOCK_FETCH_TIME_IN_MS: u64 = 250;

    pub struct MockRelayServer<'a> {
        mock_store: &'a ValidatorStore,
    }

    impl MockRelayServer<'_> {
        pub fn new(mock_store: &ValidatorStore) -> MockRelayServer {
            MockRelayServer { mock_store }
        }

        pub async fn fetch_latest_block_info(&self) -> Result<Block, GDEXError> {
            let result = self
                .mock_store
                .last_block_store
                .read(0)
                .await
                .expect("Error fetching last block store")
                // Ok to unwrap as the db should always contain a latest block
                .unwrap();

            // sleep while testing to simulate processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(MOCK_FETCH_TIME_IN_MS)).await;

            Ok(result)
        }

        pub async fn fetch_block_info(&self, block_number: BlockNumber) -> Result<BlockInfo, GDEXError> {
            let next_block_info = self
                .mock_store
                .block_info_store
                .read(block_number)
                .await
                .expect("Expected the next block info to exist")
                .unwrap();

            // sleep while testing to simulate processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(MOCK_FETCH_TIME_IN_MS)).await;

            Ok(next_block_info)
        }

        pub async fn fetch_block(&self, block_number: BlockNumber) -> Result<Block, GDEXError> {
            let next_block = self
                .mock_store
                .block_store
                .read(block_number)
                .await
                .expect("Expected the next block to exist")
                .unwrap();

            // sleep while testing to simulate processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(MOCK_FETCH_TIME_IN_MS)).await;

            Ok(next_block)
        }
    }

    pub struct MockCatchupManger {
        network_processed_block_number: BlockNumber,
        catchup_processed_block_number: BlockNumber,
        network_block_info: Vec<BlockInfo>,
        chunk_size: u64,
    }

    impl MockCatchupManger {
        pub fn new(chunk_size: u64) -> Self {
            MockCatchupManger {
                network_block_info: Vec::new(),
                network_processed_block_number: 0,
                catchup_processed_block_number: 0,
                chunk_size,
            }
        }

        /// This fetches the latest block from a mock relay server, e.g. from a coincident nodes validator store
        async fn fetch_latest_block(&mut self, mock_server: &MockRelayServer<'_>) -> Result<(), GDEXError> {
            let latest_block = mock_server.fetch_latest_block_info().await?.clone();

            let latest_block_number = latest_block.block_number;
            for block_number in self.network_processed_block_number..latest_block_number {
                let next_block_info = mock_server.fetch_block_info(block_number).await?;
                self.network_block_info.push(next_block_info);
            }
            self.network_block_info
                .sort_by(|a, b| a.block_number.cmp(&b.block_number));
            self.network_processed_block_number = latest_block_number;
            Ok(())
        }

        /// This catches up to the latest block from a mock relay server, e.g. from a coincident nodes validator store
        pub async fn catchup_to_latest_block(
            &mut self,
            mock_server: &MockRelayServer<'_>,
            new_state: &Arc<ValidatorState>,
        ) -> Result<(), GDEXError> {
            while self.network_processed_block_number != self.catchup_processed_block_number {
                // TODO - update to warn after finishing testing
                println!("Processing until block {}", self.catchup_processed_block_number);
                let prev_chunk_start = self.catchup_processed_block_number;

                self.catchup_processed_block_number = std::cmp::min(
                    self.catchup_processed_block_number + self.chunk_size,
                    (self.network_block_info.len())
                        .try_into()
                        .expect("Problem initializing network block info"),
                );
                let next_chunk = (self.network_block_info
                    [prev_chunk_start as usize..self.catchup_processed_block_number as usize])
                    .iter();

                // iterate over the next chunk and process each block
                for next_block_info in next_chunk {
                    let next_block_number = next_block_info.block_number;
                    // if we have already received this block, skip forward
                    if new_state
                        .validator_store
                        .block_info_store
                        .read(next_block_number)
                        .await
                        .expect("Failed to check local state for block info {next_block_info.block_number}")
                        .is_some()
                    {
                        println!("Block {} already exists in the store, continuing", next_block_number);
                        continue;
                    }

                    let next_block = mock_server.fetch_block(next_block_number).await?.clone();

                    // TODO - checks to gaurentee security of downloaded data, like txn hash matches block hash

                    new_state
                        .validator_store
                        .write_latest_block(next_block_info.block_certificate.clone(), next_block.transactions)
                        .await;
                }
            }
            Ok(())
        }

        pub async fn catchup(
            &mut self,
            mock_server: &MockRelayServer<'_>,
            new_state: &Arc<ValidatorState>,
        ) -> Result<(), GDEXError> {
            loop {
                self.fetch_latest_block(mock_server).await?;
                println!(
                    "Catching up to block {} after processing block {}",
                    self.network_processed_block_number, self.catchup_processed_block_number
                );
                self.catchup_to_latest_block(mock_server, new_state).await?;
                // fetch the latest block a second time and check that it matches after doing catch-up
                self.fetch_latest_block(mock_server).await?;
                // TODO - update state here
                if self.network_processed_block_number == self.catchup_processed_block_number {
                    break;
                }
            }
            Ok(())
        }
    }

    const MAX_CHUNK_SIZE: u64 = 1_000;
    impl Default for MockCatchupManger {
        fn default() -> Self {
            Self::new(MAX_CHUNK_SIZE)
        }
    }
}

pub struct CatchupManager {
    network_processed_block_number: BlockNumber,
    catchup_processed_block_number: BlockNumber,
}

impl CatchupManager {
    const SLEEP_PER_QUERY: u64 = 100;

    pub fn new() -> Self {
        CatchupManager {
            network_processed_block_number: 0,
            catchup_processed_block_number: 0,
        }
    }

    pub async fn catchup_to_latest_block(
        &mut self,
        mock_server: &mock_catchup_manager::MockRelayServer<'_>,
        new_state: &Arc<ValidatorState>,
    ) -> Result<(), GDEXError> {
        loop {
            let latest_block_network = mock_server.fetch_latest_block_info().await?.clone();
            let latest_block_local = new_state
                .validator_store
                .last_block_store
                .read(0)
                .await
                .expect("Error fetching last block store")
                // Ok to unwrap as the db should always contain a latest block
                .unwrap();

            println!("latest_block_network={:?}", latest_block_network.block_number);
            println!("latest_block_local={:?}", latest_block_local.block_number);
            if latest_block_local.block_number == latest_block_network.block_number {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(Self::SLEEP_PER_QUERY)).await;
        }
        Ok(())
    }
}

impl Default for CatchupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod suite_catchup_tests {
    use super::mock_catchup_manager::*;
    use super::*;
    use crate::{client, validator::spawner::ValidatorSpawner};
    use gdex_types::{
        account::account_test_functions::generate_keypair_vec,
        proto::{TransactionProto, TransactionsClient},
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };
    use std::{io, path::Path};
    use tracing::info;

    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn other_mock_catchup_fifth_node() {
        // telemetry_subscribers::init_for_testing();

        let dir = "../.proto";
        let temp_dir = tempfile::tempdir().unwrap().path().to_path_buf();
        let path = Path::new(dir).to_path_buf();

        println!("Spawning validator 0");
        let mut spawner_0 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-0".to_string(),
        );

        let _handler_0 = spawner_0.spawn_validator().await;
        spawner_0.unhalt_validator();

        println!("Spawning validator 1");
        let mut spawner_1 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-1".to_string(),
        );
        let _handler_1 = spawner_1.spawn_validator().await;
        spawner_1.unhalt_validator();

        println!("Spawning validator 2");
        let mut spawner_2 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-2".to_string(),
        );
        let _handler_2 = spawner_2.spawn_validator().await;
        spawner_2.unhalt_validator();

        println!("Spawning validator 3");
        let mut spawner_3 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-3".to_string(),
        );
        let _handler_3 = spawner_3.spawn_validator().await;
        spawner_3.unhalt_validator();

        println!("Sending transactions");
        let key_file = path.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        println!("Connecting network client to address={:?}", address);

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        // send 1_000 transactions to the local cluster
        println!("Sending transactions to cluster");
        tokio::spawn(async move {
            let mut i = 1;
            let mut signed_transactions = Vec::new();
            let n_transactions_to_submit = 1_000_000;
            while i < n_transactions_to_submit + 1 {
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
        });

        println!("Sleeping 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        println!("Launching node 4");
        let mut spawner_4 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-4".to_string(),
        );
        let _handler_4 = spawner_4.spawn_validator().await;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let validator_store = &spawner_1
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        let new_validator_state = spawner_4.get_validator_state().unwrap();

        let mut mock_catchup_manager = CatchupManager::new();
        let mock_server = MockRelayServer::new(validator_store);
        mock_catchup_manager
            .catchup_to_latest_block(&mock_server, &new_validator_state)
            .await
            .unwrap();
        println!("Catchup done");
    }

    // TODO - use Paul cluster setup
    #[ignore]
    #[tokio::test(flavor = "multi_thread")]
    pub async fn mock_catchup_fifth_node() {
        // telemetry_subscribers::init_for_testing();

        let dir = "../.proto";
        let temp_dir = tempfile::tempdir().unwrap().path().to_path_buf();
        let path = Path::new(dir).to_path_buf();

        println!("Spawning validator 0");
        let mut spawner_0 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-0".to_string(),
        );

        let _handler_0 = spawner_0.spawn_validator().await;
        spawner_0.unhalt_validator();

        println!("Spawning validator 1");
        let mut spawner_1 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-1".to_string(),
        );
        let _handler_1 = spawner_1.spawn_validator().await;
        spawner_1.unhalt_validator();

        println!("Spawning validator 2");
        let mut spawner_2 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-2".to_string(),
        );
        let _handler_2 = spawner_2.spawn_validator().await;
        spawner_2.unhalt_validator();

        println!("Spawning validator 3");
        let mut spawner_3 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-3".to_string(),
        );
        let _handler_3 = spawner_3.spawn_validator().await;
        spawner_3.unhalt_validator();

        println!("Sending transactions");
        let key_file = path.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        println!("Connecting network client to address={:?}", address);

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        // send 1_000 transactions to the local cluster
        println!("Sending transactions to cluster");
        tokio::spawn(async move {
            let mut i = 1;
            let mut signed_transactions = Vec::new();
            let n_transactions_to_submit = 1_000_000;
            while i < n_transactions_to_submit + 1 {
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
        });

        println!("Sleeping 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        println!("Launching node 4");
        let mut spawner_4 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-4".to_string(),
        );
        let _handler_4 = spawner_4.spawn_validator().await;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let validator_store = &spawner_1
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        let new_validator_state = spawner_4.get_validator_state().unwrap();

        let mut mock_catchup_manager = MockCatchupManger::new(5);
        let mock_server = MockRelayServer::new(validator_store);
        mock_catchup_manager
            .catchup_to_latest_block(&mock_server, &new_validator_state)
            .await
            .unwrap();
        println!("Catchup done");
    }
}
