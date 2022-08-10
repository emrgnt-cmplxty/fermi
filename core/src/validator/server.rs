//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority_server.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use crate::{
    client,
    config::node::NodeConfig,
    validator::{consensus_adapter::ConsensusAdapter, state::ValidatorState},
};
use anyhow::anyhow;
use async_trait::async_trait;
use gdex_server::api::{ValidatorAPI, ValidatorAPIServer};
use gdex_types::{crypto::KeypairTraits, transaction::SignedTransaction};
use multiaddr::Multiaddr;
use narwhal_executor::SubscriberError;
use narwhal_types::{TransactionProto, TransactionsClient};
use prometheus::Registry;
use std::{io, sync::Arc, time::Duration};
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tracing::info;

const MIN_BATCH_SIZE: u64 = 1000;
const MAX_DELAY_MILLIS: u64 = 5_000; // 5 sec

/// Contains and orchestrates a tokio handle where the validator server runs
pub struct ValidatorServerHandle {
    tx_cancellation: tokio::sync::oneshot::Sender<()>,
    local_addr: Multiaddr,
    handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

impl ValidatorServerHandle {
    pub async fn join(self) -> Result<(), std::io::Error> {
        // Note that dropping `self.complete` would terminate the server.
        self.handle
            .await?
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub async fn kill(self) -> Result<(), std::io::Error> {
        self.tx_cancellation
            .send(())
            .map_err(|_e| std::io::Error::new(io::ErrorKind::Other, "could not send cancellation signal!"))?;
        self.handle
            .await?
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn address(&self) -> &Multiaddr {
        &self.local_addr
    }
}

/// Can spawn a validator server handle at the internal address
/// the server handle contains a validator api (grpc) that exposes a validator service
pub struct ValidatorServer {
    address: Multiaddr,
    pub state: Arc<ValidatorState>,
    consensus_adapter: ConsensusAdapter,
    pub min_batch_size: u64,
    pub max_delay: Duration,
}

impl ValidatorServer {
    pub fn new(address: Multiaddr, state: Arc<ValidatorState>, consensus_address: Multiaddr) -> Self {
        let consensus_client =
            TransactionsClient::new(client::connect_lazy(&consensus_address).expect("Failed to connect to consensus"));
        let consensus_adapter = ConsensusAdapter {
            consensus_client,
            consensus_address,
        };
        Self {
            address,
            state,
            consensus_adapter,
            min_batch_size: MIN_BATCH_SIZE,
            max_delay: Duration::from_millis(MAX_DELAY_MILLIS),
        }
    }

    pub async fn spawn(self) -> Result<ValidatorServerHandle, io::Error> {
        let address = self.address.clone();
        self.spawn_with_bind_address(address).await
    }

    pub async fn spawn_with_bind_address(self, address: Multiaddr) -> Result<ValidatorServerHandle, io::Error> {
        let mut server = crate::config::server::ServerConfig::new()
            .server_builder()
            .add_service(ValidatorAPIServer::new(ValidatorService {
                state: self.state,
                consensus_adapter: Arc::new(Mutex::new(self.consensus_adapter)),
            }))
            .bind(&address)
            .await
            .unwrap();
        let local_addr = server.local_addr().to_owned();
        info!("Listening to traffic on {local_addr}");
        let handle = ValidatorServerHandle {
            tx_cancellation: server.take_cancel_handle().unwrap(),
            local_addr,
            handle: tokio::spawn(server.serve()),
        };
        Ok(handle)
    }
}

/// Handles communication with consensus and resulting validator state updates
pub struct ValidatorService {
    state: Arc<ValidatorState>,
    pub consensus_adapter: Arc<Mutex<ConsensusAdapter>>,
}

impl ValidatorService {
    /// Spawn all the subsystems run by a gdex valdiator: a consensus node, a gdex valdiator server,
    /// and a consensus listener bridging the consensus node and the gdex valdiator.
    pub async fn new(
        config: &NodeConfig,
        state: Arc<ValidatorState>,
        prometheus_registry: &Registry,
    ) -> anyhow::Result<Self> {
        let (tx_consensus_to_sui, rx_consensus_to_sui) = channel(1_000);
        // let (tx_sui_to_consensus, rx_sui_to_consensus) = channel(1_000);

        // Spawn the consensus node of this authority.
        let consensus_config = config
            .consensus_config()
            .ok_or_else(|| anyhow!("Validator is missing consensus config"))?;
        let consensus_keypair = config.key_pair().copy();
        let consensus_name = consensus_keypair.public().clone();
        let consensus_store = narwhal_node::NodeStorage::reopen(consensus_config.db_path());

        println!(
            "spawning a primary with parameters={:?}",
            consensus_config.narwhal_config()
        );
        narwhal_node::Node::spawn_primary(
            consensus_keypair,
            config.genesis()?.narwhal_committee(),
            &consensus_store,
            consensus_config.narwhal_config().to_owned(),
            /* consensus */ true, // Indicate that we want to run consensus.
            /* execution_state */ Arc::clone(&state),
            /* tx_confirmation */ tx_consensus_to_sui,
            prometheus_registry,
        )
        .await?;

        narwhal_node::Node::spawn_workers(
            consensus_name,
            /* ids */ vec![0], // We run a single worker with id '0'.
            config.genesis()?.narwhal_committee(),
            &consensus_store,
            consensus_config.narwhal_config().to_owned(),
            prometheus_registry,
        );

        println!("connecting the adapter at address={:?}", consensus_config.address());
        let consensus_client = TransactionsClient::new(
            client::connect_lazy(consensus_config.address()).expect("Failed to connect to consensus"),
        );
        let consensus_adapter = ConsensusAdapter {
            consensus_client,
            consensus_address: consensus_config.address().to_owned(),
        };

        // Create a new task to listen to received transactions
        tokio::spawn(async move {
            Self::analyze(rx_consensus_to_sui).await;
        });

        Ok(Self {
            state,
            consensus_adapter: Arc::new(Mutex::new(consensus_adapter)),
        })
    }

    /// Receives an ordered list of certificates and apply any application-specific logic.
    #[allow(clippy::type_complexity)]
    async fn analyze(mut rx_output: Receiver<(Result<Vec<u8>, SubscriberError>, Vec<u8>)>) {
        loop {
            while let Some(_message) = rx_output.recv().await {
                // NOTE: Notify the user that its transaction has been processed.
                println!("the transaction was received and processed....")
            }
        }
    }

    async fn handle_transaction(
        consensus_adapter: Arc<Mutex<ConsensusAdapter>>,
        state: Arc<ValidatorState>,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<SignedTransaction>, tonic::Status> {
        println!("handling transaction on the validator service");

        let transaction = request.into_inner();

        transaction
            .verify()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        let transaction_proto = TransactionProto {
            transaction: transaction.serialize().unwrap().into(),
        };

        let result = consensus_adapter
            .lock()
            .await
            .consensus_client
            .submit_transaction(transaction_proto)
            .await
            .unwrap();

        println!("result = {:?}", result);

        state
            .handle_transaction(&transaction)
            // .instrument(span)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(transaction))
    }
}

/// Spawns a tonic grpc which parses incoming transactions and forwards them to the handle_transaction method of ValidatorService
#[async_trait]
impl ValidatorAPI for ValidatorService {
    async fn transaction(
        &self,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<SignedTransaction>, tonic::Status> {
        let state = self.state.clone();
        let consensus_adapter = self.consensus_adapter.clone();

        // Spawns a task which handles the transaction. The task will unconditionally continue
        // processing in the event that the client connection is dropped.
        tokio::spawn(async move { Self::handle_transaction(consensus_adapter, state, request).await })
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod test_validator_server {
    use super::*;
    use crate::{
        builder::genesis_state::GenesisStateBuilder,
        client::{ClientAPI, NetworkValidatorClient},
        genesis_ceremony::VALIDATOR_FUNDING_AMOUNT,
    };
    use gdex_controller::master::MasterController;
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair, ValidatorPubKeyBytes},
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };

    async fn spawn_validator_server() -> Result<ValidatorServerHandle, io::Error> {
        let master_controller = MasterController::default();

        let key: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key = ValidatorPubKeyBytes::from(key.public());
        let secret = Arc::pin(key);

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: public_key.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };
        let network_address = validator.network_address.clone();

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();
        let validator_state = ValidatorState::new(public_key, secret, &genesis).await;
        let new_addr = utils::new_network_address();
        let validator_server = ValidatorServer::new(new_addr.clone(), Arc::new(validator_state), network_address);
        validator_server.spawn().await
    }

    #[tokio::test]
    pub async fn server_init() {
        spawn_validator_server().await.unwrap();
    }

    #[tokio::test]
    pub async fn server_process_transaction() {
        let handle_result = spawn_validator_server().await;
        let handle = handle_result.unwrap();
        let client = NetworkValidatorClient::connect_lazy(&handle.address()).unwrap();

        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let _resp1 = client
            .handle_transaction(signed_transaction)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    }

    #[tokio::test]
    pub async fn multiple_server_init() {
        let master_controller = MasterController::default();

        let key_0: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key_0 = ValidatorPubKeyBytes::from(key_0.public());
        let secret_0 = Arc::pin(key_0);

        let key_1: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let public_key_1 = ValidatorPubKeyBytes::from(key_1.public());
        let secret_1 = Arc::pin(key_1);

        println!("public_key_0={:?}", public_key_0);
        println!("public_key_1={:?}", public_key_1);

        let validator_0 = ValidatorInfo {
            name: "0".into(),
            public_key: public_key_0.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let validator_1 = ValidatorInfo {
            name: "1".into(),
            public_key: public_key_1.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator_0)
            .add_validator(validator_1);

        let genesis = builder.build();
        let validator_state_0 = ValidatorState::new(public_key_0, secret_0, &genesis).await;
        let validator_state_1 = ValidatorState::new(public_key_1, secret_1, &genesis).await;

        let validator_server_0 = ValidatorServer::new(
            utils::new_network_address(),
            Arc::new(validator_state_0),
            utils::new_network_address(),
        );
        let validator_server_1 = ValidatorServer::new(
            utils::new_network_address(),
            Arc::new(validator_state_1),
            utils::new_network_address(),
        );

        validator_server_0.spawn().await.unwrap();
        validator_server_1.spawn().await.unwrap();
    }
}
