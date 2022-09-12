//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

/// This module provides functionality to fetch the latest block given a mock relay sever
/// moreover, the module then iterates until the latest block number of the passed validator  
/// state matches the mock block number of the newly generated validator store
use crate::validator::state::ValidatorState;
use gdex_types::{
    error::GDEXError,
    proto::{RelayerClient, RelayerGetLatestBlockInfoRequest},
};
use std::sync::Arc;
use tonic::transport::Channel;
use tracing::log::info;
const SLEEP_PER_QUERY: u64 = 100;

#[cfg(any(test, feature = "testing"))]
pub mod mock_catchup_manager {
    use super::SLEEP_PER_QUERY;
    use crate::validator::state::{ValidatorState, ValidatorStore};
    use gdex_types::{
        block::{Block, BlockInfo, BlockNumber},
        error::GDEXError,
    };
    use std::sync::Arc;
    use tracing::info;

    const MOCK_FETCH_TIME_IN_MS: u64 = 250;

    pub struct MockRelayServer<'a> {
        mock_store: &'a ValidatorStore,
    }

    impl MockRelayServer<'_> {
        pub fn new(mock_store: &ValidatorStore) -> MockRelayServer {
            MockRelayServer { mock_store }
        }

        pub async fn fetch_latest_block_info(&self) -> Result<BlockInfo, GDEXError> {
            let result = self
                .mock_store
                .process_block_store
                .last_block_info_store
                .read(0)
                .await
                .expect("Error fetching from the last block store")
                .expect("Latest block info was unexpectedly empty");

            // sleep while testing to simulate processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(MOCK_FETCH_TIME_IN_MS)).await;

            Ok(result)
        }

        pub async fn fetch_block_info(&self, block_number: BlockNumber) -> Result<BlockInfo, GDEXError> {
            let next_block_info = self
                .mock_store
                .process_block_store
                .block_info_store
                .read(block_number)
                .await
                .expect("Error fetching from the block info store")
                .expect("Block Info {block_number} was unexpectedly empty");

            // sleep while testing to simulate processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(MOCK_FETCH_TIME_IN_MS)).await;

            Ok(next_block_info)
        }

        pub async fn fetch_block(&self, block_number: BlockNumber) -> Result<Block, GDEXError> {
            let next_block = self
                .mock_store
                .process_block_store
                .block_store
                .read(block_number)
                .await
                .expect("Error fetching from the block store")
                .expect("Block {block_number} was unexpectedly empty");

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
            let latest_block = mock_server.fetch_latest_block_info().await?;

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
                info!("Processing until block {}", self.catchup_processed_block_number);
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
                        .process_block_store
                        .block_info_store
                        .read(next_block_number)
                        .await
                        .expect("Failed to check local state for block info {next_block_info.block_number}")
                        .is_some()
                    {
                        info!("Block {} already exists in the store, continuing", next_block_number);
                        continue;
                    }

                    let next_block = mock_server.fetch_block(next_block_number).await?.clone();

                    // TODO - checks to gaurentee security of downloaded data, like txn hash matches block hash

                    new_state
                        .validator_store
                        .write_latest_block(next_block.block_certificate.clone(), next_block.transactions)
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
                info!(
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

        pub async fn catchup_narwhal_mediated(
            &mut self,
            mock_server: &MockRelayServer<'_>,
            new_state: &Arc<ValidatorState>,
        ) -> Result<(), GDEXError> {
            info!("Catching up until fetched block matches latest network block");
            loop {
                let latest_block_network = mock_server.fetch_latest_block_info().await?.clone();
                let latest_block_local = new_state
                    .validator_store
                    .process_block_store
                    .last_block_info_store
                    .read(0)
                    .await
                    .expect("Error fetching last block store");

                if let Some(latest_block_local) = latest_block_local {
                    info!(
                        "Catching up from block={} to block={}",
                        latest_block_local.block_number, latest_block_network.block_number
                    );
                    if latest_block_local.block_number == latest_block_network.block_number {
                        break;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_PER_QUERY)).await;
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

// TODO - implement catch-up logic inside of node/main.rs
// TODO - think of smart way to address finding relayer address. This can likely be handled as follows:
// Loop over a provided list of relayers and find the best relayer(s) to facilitate catching-up
pub struct CatchupManager {
    relayer_client: RelayerClient<Channel>,
    validator_state: Arc<ValidatorState>,
}

impl CatchupManager {
    pub fn new(relayer_client: RelayerClient<Channel>, validator_state: Arc<ValidatorState>) -> Self {
        CatchupManager {
            relayer_client,
            validator_state,
        }
    }

    pub async fn catchup_narwhal_mediated(&mut self) -> Result<(), GDEXError> {
        info!("Catching up until fetched block matches latest network block");
        loop {
            let latest_block_info_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {});
            let latest_block_info_response = self
                .relayer_client
                .get_latest_block_info(latest_block_info_request)
                .await;
            let block_info_returned = latest_block_info_response.unwrap().into_inner().block_info.unwrap();

            let latest_block_local = self
                .validator_state
                .validator_store
                .process_block_store
                .last_block_info_store
                .read(0)
                .await
                .expect("Error fetching last block store");

            if let Some(latest_block_local) = latest_block_local {
                info!(
                    "Catching up from block={} to block={}",
                    latest_block_local.block_number, block_info_returned.block_number
                );
                if latest_block_local.block_number == block_info_returned.block_number {
                    break;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_PER_QUERY)).await;
        }

        Ok(())
    }
}
