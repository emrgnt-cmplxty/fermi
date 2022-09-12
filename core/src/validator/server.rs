//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority_server.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use crate::{
    config::node::NodeConfig,
    validator::{consensus_adapter::ConsensusAdapter, restarter::NodeRestarter, state::ValidatorState},
};
use anyhow::anyhow;
use async_trait::async_trait;
use futures::StreamExt;
use gdex_types::{
    crypto::KeypairTraits,
    error::GDEXError,
    proto::{Empty, TransactionSubmitter, TransactionSubmitterServer},
    transaction::{ConsensusTransaction, SignedTransaction},
};
use multiaddr::Multiaddr;
use narwhal_config::Committee as ConsensusCommittee;
use narwhal_consensus::ConsensusOutput;
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use narwhal_executor::{ExecutionIndices, SerializedTransaction, SubscriberError};
use narwhal_types::TransactionProto as ConsensusTransactionWrapper;
use prometheus::Registry;
use std::{io, sync::Arc, time::SystemTime};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tracing::{info, trace};

// constants
// frequency of orderbook depth writes (rounds)
type ExecutionResult = Result<(), GDEXError>;
type HandledTransaction = Result<(ConsensusOutput, ExecutionIndices, ExecutionResult), SubscriberError>;

/// Contains and orchestrates a tokio handle where the validator server runs
pub struct ValidatorServerHandle {
    local_addr: Multiaddr,
    handle: JoinHandle<()>,
    adapter: Arc<ConsensusAdapter>,
}

impl ValidatorServerHandle {
    pub fn validator_address(&self) -> &Multiaddr {
        &self.local_addr
    }

    pub fn get_adapter(&self) -> Arc<ConsensusAdapter> {
        Arc::clone(&self.adapter)
    }

    pub fn get_handle(self) -> JoinHandle<()> {
        self.handle
    }
}

/// Can spawn a validator server handle at the internal validator_address
/// the server handle contains a validator api (grpc) that exposes a validator service
pub struct ValidatorServer {
    validator_address: Multiaddr,
    state: Arc<ValidatorState>,
    consensus_adapter: Arc<ConsensusAdapter>,
}

impl ValidatorServer {
    pub fn new(
        validator_address: Multiaddr,
        state: Arc<ValidatorState>,
        consensus_addresses: Vec<Multiaddr>,
        tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
    ) -> Self {
        let consensus_adapter = Arc::new(ConsensusAdapter::new(consensus_addresses, tx_reconfigure_consensus));

        Self {
            validator_address,
            state,
            consensus_adapter,
        }
    }

    // TODO this is kinda dumb
    pub async fn spawn(self) -> Result<ValidatorServerHandle, io::Error> {
        let validator_address = self.validator_address.clone();
        info!(
            "Calling spawn to produce a the validator server with port validator_address = {:?}",
            validator_address
        );
        self.run(validator_address).await
    }

    pub async fn run(self, validator_address: Multiaddr) -> Result<ValidatorServerHandle, io::Error> {
        let server = crate::config::server::ServerConfig::new()
            .server_builder()
            .add_service(TransactionSubmitterServer::new(ValidatorService {
                state: self.state.clone(),
                consensus_adapter: self.consensus_adapter.clone(),
            }))
            .bind(&validator_address)
            .await
            .unwrap();
        let local_addr = server.local_addr().to_owned();
        info!("Listening to traffic on {local_addr}");
        let handle = ValidatorServerHandle {
            local_addr,
            handle: tokio::spawn(server.serve()),
            adapter: self.consensus_adapter,
        };
        Ok(handle)
    }
}

/// Handles communication with consensus and resulting validator state updates
pub struct ValidatorService {
    state: Arc<ValidatorState>,
    consensus_adapter: Arc<ConsensusAdapter>,
}

impl ValidatorService {
    /// Spawn all the subsystems run by a gdex valdiator: a consensus node, a gdex valdiator server,
    /// and a consensus listener bridging the consensus node and the gdex valdiator.
    pub async fn spawn_narwhal(
        config: &NodeConfig,
        state: Arc<ValidatorState>,
        prometheus_registry: &Registry,
        rx_reconfigure_consensus: Receiver<(ConsensusKeyPair, ConsensusCommittee)>,
    ) -> anyhow::Result<Vec<JoinHandle<()>>> {
        let (tx_consensus_to_gdex, rx_consensus_to_gdex) = channel(1_000);
        // Spawn the consensus node of this authority.
        let consensus_config = config
            .consensus_config()
            .ok_or_else(|| anyhow!("Validator is missing consensus config"))?;
        let consensus_keypair = config.key_pair().copy();
        let consensus_committee = config.genesis()?.narwhal_committee().load();
        let consensus_worker_cache = config.genesis()?.narwhal_worker_cache();
        let consensus_execution_state = Arc::clone(&state);
        let consensus_storage_base_path = consensus_config.db_path().to_path_buf();
        let consensus_parameters = consensus_config.narwhal_config().to_owned();

        info!("consensus_committee = {:?}", consensus_committee);

        let registry = prometheus_registry.clone();
        let restarter_handle = tokio::spawn(async move {
            NodeRestarter::watch(
                consensus_keypair,
                &*consensus_committee,
                consensus_worker_cache,
                consensus_storage_base_path,
                consensus_execution_state,
                consensus_parameters,
                rx_reconfigure_consensus,
                tx_consensus_to_gdex,
                &registry,
            )
            .await
        });
        // Create a new task to listen to received transactions
        let post_process_handle = tokio::spawn(async move {
            Self::post_process(rx_consensus_to_gdex, Arc::clone(&state)).await;
        });

        Ok(vec![restarter_handle, post_process_handle])
    }

    /// Receives an ordered list of certificates and apply any application-specific logic.
    async fn post_process(
        mut rx_output: Receiver<(HandledTransaction, SerializedTransaction)>,
        validator_state: Arc<ValidatorState>,
    ) {
        // TODO load the actual last block
        let mut serialized_txns_buf = Vec::new();
        let store = &validator_state.validator_store;
        let metrics = &validator_state.metrics;
        let master_controller = &validator_state.master_controller;
        loop {
            while let Some(message) = rx_output.recv().await {
                trace!("Received a finalized consensus transaction for post processing",);
                let (result, serialized_txn) = message;
                match result {
                    Ok((consensus_output, execution_indices, execution_result)) => {
                        serialized_txns_buf.push((serialized_txn, execution_result));

                        // if next_transaction_index == 0 then the block is complete and we may write-out
                        if execution_indices.next_transaction_index == 0 {
                            // subtract round look-back from the latest round to get block number
                            let round_number = consensus_output.certificate.header.round;

                            let num_txns = serialized_txns_buf.len();
                            trace!("Processing result from {round_number} with {num_txns} transactions");
                            store.prune();
                            // write-out the new block to the validator store
                            let (block, block_info) = store
                                .write_latest_block(consensus_output.certificate, serialized_txns_buf.clone())
                                .await;
                            metrics.process_end_of_block(block, block_info);
                            let block_number = store.block_number.load(std::sync::atomic::Ordering::SeqCst);
                            master_controller
                                .process_end_of_block(&store.process_block_store, block_number)
                                .await;
                            serialized_txns_buf.clear();
                        }
                    }
                    Err(e) => info!("{:?}", e), // TODO
                }
                // NOTE: Notify the user that its transaction has been processed.
            }
        }
    }

    async fn handle_transaction(
        consensus_adapter: Arc<ConsensusAdapter>,
        state: Arc<ValidatorState>,
        signed_transaction: SignedTransaction,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let start = SystemTime::now();
        trace!("Handling a new transaction with ValidatorService",);
        state.metrics.transactions_received.inc();

        signed_transaction
            .verify_signature()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        // check recent block hash is valid
        // TODO seems maybe problematic to do this just here?
        // TODO change this to err flow
        // TODO there is a ton of contention here
        let recent_block_digest = signed_transaction
            .get_recent_block_digest()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        if !state.validator_store.cache_contains_block_digest(&recent_block_digest) {
            state.metrics.transactions_received_failed.inc();
            cfg_if::cfg_if! {
                if #[cfg(feature = "benchmark")] {
                    trace!("A submitted transaction digest was invalid");
                } else {
                    return Err(tonic::Status::internal("Invalid recent certificate digest"));
                }
            }
        }

        // check transaction is not a duplicate
        let transaction = signed_transaction
            .get_transaction()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        if state.validator_store.cache_contains_transaction(transaction) {
            state.metrics.transactions_received_failed.inc();
            // TODO - find cleaner way to represent this logic
            // TODO - make sure benchmark flag is removed from node Cargo.toml in the future
            let digest = "TEST"; // TODO impl txn to string
            cfg_if::cfg_if! {
                if #[cfg(feature = "benchmark")] {
                    trace!("Duplicate transaction id = {}", digest);
                } else {
                    return Err(tonic::Status::internal("Duplicate transaction ".to_owned() + &digest));
                }
            }
        }

        // submit transaction
        let serialized_consensus_transaction = ConsensusTransaction::new(&signed_transaction)
            .serialize()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        let consensus_transaction_wrapper = ConsensusTransactionWrapper {
            transaction: serialized_consensus_transaction.into(),
        };

        consensus_adapter
            .submit_transaction(consensus_transaction_wrapper)
            .await?;

        state
            .handle_pre_consensus_transaction(&signed_transaction)
            // .instrument(span)
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let processing_time_in_micros: u64 = SystemTime::now()
            .duration_since(start)
            .unwrap()
            .as_micros()
            .try_into()
            .unwrap();

        state
            .metrics
            .transaction_rec_latency_in_micros
            .observe(processing_time_in_micros as f64);

        Ok(tonic::Response::new(Empty {}))
    }
}

/// Spawns a tonic grpc which parses incoming transactions and forwards them to the handle_transaction method of ValidatorService
#[async_trait]
impl TransactionSubmitter for ValidatorService {
    async fn submit_transaction(
        &self,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        trace!("Handling a new transaction with a ValidatorService ValidatorAPI",);
        let signed_transaction = request.into_inner();
        let state = self.state.clone();
        let consensus_adapter = self.consensus_adapter.clone();

        // Spawns a task which handles the transaction. The task will unconditionally continue
        // processing in the event that the client connection is dropped.
        tokio::spawn(async move { Self::handle_transaction(consensus_adapter, state, signed_transaction).await })
            .await
            .unwrap()
    }

    async fn submit_transaction_stream(
        &self,
        request: tonic::Request<tonic::Streaming<SignedTransaction>>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let mut signed_transactions = request.into_inner();
        trace!("Handling a new transaction stream with a ValidatorService ValidatorAPI",);

        while let Some(Ok(signed_transaction)) = signed_transactions.next().await {
            trace!("Streaming a new transaction with a ValidatorService ValidatorAPI",);

            let state = self.state.clone();
            let consensus_adapter = self.consensus_adapter.clone();

            tokio::spawn(async move { Self::handle_transaction(consensus_adapter, state, signed_transaction).await })
                .await
                .unwrap()?;
        }
        Ok(tonic::Response::new(Empty {}))
    }
}

#[cfg(test)]
mod test_validator_server {
    use super::*;
    use crate::{
        builder::genesis_state::GenesisStateBuilder,
        client,
        genesis_ceremony::{VALIDATOR_BALANCE, VALIDATOR_FUNDING_AMOUNT},
        validator::metrics::ValidatorMetrics,
    };
    use gdex_controller::master::MasterController;
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair, ValidatorPubKeyBytes},
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        proto::TransactionSubmitterClient,
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };

    async fn spawn_test_validator_server() -> Result<ValidatorServerHandle, io::Error> {
        let master_controller = MasterController::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);
        let consensus_addresses = vec![utils::new_network_address()];
        let validator = ValidatorInfo {
            name: "0".into(),
            public_key,
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: consensus_addresses.clone(),
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();
        let registry = Registry::default();
        let metrics = Arc::new(ValidatorMetrics::new(&registry));
        let validator_state = ValidatorState::new(public_key, secret, &genesis, &store_path, metrics);
        let new_addr = utils::new_network_address();
        let (tx_reconfigure_consensus, _rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);
        let validator_server = ValidatorServer::new(
            new_addr.clone(),
            Arc::new(validator_state),
            consensus_addresses,
            tx_reconfigure_consensus,
        );
        validator_server.spawn().await
    }

    #[tokio::test]
    pub async fn server_test_init() {
        spawn_test_validator_server().await.unwrap();
    }

    #[tokio::test]
    pub async fn server_process_transaction() {
        let handle_result = spawn_test_validator_server().await;
        let handle = handle_result.unwrap();
        let mut client = TransactionSubmitterClient::new(
            client::connect_lazy(handle.validator_address()).expect("Failed to connect to consensus"),
        );

        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver, 10);

        let _resp1 = client
            .submit_transaction(signed_transaction)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    }

    #[tokio::test]
    pub async fn spawn() {
        let master_controller = MasterController::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);
        let consensus_addresses = vec![utils::new_network_address()];

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key,
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: consensus_addresses.clone(),
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let store_path = tempfile::tempdir()
            .expect("Failed to open temporary directory")
            .into_path();

        let registry = Registry::default();
        let metrics = Arc::new(ValidatorMetrics::new(&registry));
        let validator_state = ValidatorState::new(public_key, secret, &genesis, &store_path, metrics);
        let new_addr = utils::new_network_address();
        let (tx_reconfigure_consensus, _rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);
        let validator_server = ValidatorServer::new(
            new_addr.clone(),
            Arc::new(validator_state),
            consensus_addresses,
            tx_reconfigure_consensus.clone(),
        );
        validator_server.spawn().await.unwrap();
    }
}
