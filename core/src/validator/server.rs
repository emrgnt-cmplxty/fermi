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
use std::{io, sync::Arc};
use narwhal_consensus::ConsensusOutput;
use tokio::{
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, info};


/// Contains and orchestrates a tokio handle where the validator server runs
pub struct ValidatorServerHandle {
    local_addr: Multiaddr,
    handle: JoinHandle<()>,
}

impl ValidatorServerHandle {
    pub fn address(&self) -> &Multiaddr {
        &self.local_addr
    }

    pub fn get_handle(self) -> JoinHandle<()> {
        self.handle
    }
}

/// Can spawn a validator server handle at the internal address
/// the server handle contains a validator api (grpc) that exposes a validator service
pub struct ValidatorServer {
    address: Multiaddr,
    state: Arc<ValidatorState>,
    consensus_adapter: ConsensusAdapter,
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
        }
    }

    pub async fn spawn(self) -> Result<ValidatorServerHandle, io::Error> {
        let address = self.address.clone();
        info!(
            "Calling spawn to produce a the validator server with port address = {:?}",
            address
        );
        self.spawn_with_bind_address(address).await
    }

    pub async fn spawn_with_bind_address(self, address: Multiaddr) -> Result<ValidatorServerHandle, io::Error> {
        let server = crate::config::server::ServerConfig::new()
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
            local_addr,
            handle: tokio::spawn(server.serve()),
        };
        Ok(handle)
    }
}

/// Handles communication with consensus and resulting validator state updates
pub struct ValidatorService {
    state: Arc<ValidatorState>,
    consensus_adapter: Arc<Mutex<ConsensusAdapter>>,
}

impl ValidatorService {
    /// Spawn all the subsystems run by a gdex valdiator: a consensus node, a gdex valdiator server,
    /// and a consensus listener bridging the consensus node and the gdex valdiator.
    pub async fn spawn_narwhal(
        config: &NodeConfig,
        state: Arc<ValidatorState>,
        prometheus_registry: &Registry,
    ) -> anyhow::Result<Vec<JoinHandle<()>>> {
        let (tx_consensus_to_gdex, rx_consensus_to_gdex) = channel(1_000);
        // let (tx_sui_to_consensus, rx_sui_to_consensus) = channel(1_000);

        // Spawn the consensus node of this authority.
        let consensus_config = config
            .consensus_config()
            .ok_or_else(|| anyhow!("Validator is missing consensus config"))?;
        let consensus_keypair = config.key_pair().copy();
        let consensus_name = consensus_keypair.public().clone();
        let consensus_store = narwhal_node::NodeStorage::reopen(consensus_config.db_path());

        info!(
            "Creating narwhal with committee ={}",
            config.genesis()?.narwhal_committee()
        );

        let mut primary_handles = narwhal_node::Node::spawn_primary(
            consensus_keypair,
            config.genesis()?.narwhal_committee(),
            &consensus_store,
            consensus_config.narwhal_config().to_owned(),
            /* consensus */ true, // Indicate that we want to run consensus.
            /* execution_state */ Arc::clone(&state),
            /* tx_confirmation */ tx_consensus_to_gdex,
            prometheus_registry,
        )
        .await?;

        let worker_handles = narwhal_node::Node::spawn_workers(
            consensus_name,
            /* ids */ vec![0], // We run a single worker with id '0'.
            config.genesis()?.narwhal_committee(),
            &consensus_store,
            consensus_config.narwhal_config().to_owned(),
            prometheus_registry,
        );

        // Create a new task to listen to received transactions
        let post_process = tokio::spawn(async move {
            Self::post_process(rx_consensus_to_gdex, state).await;
        });

        primary_handles.extend(worker_handles);
        primary_handles.push(post_process);
        Ok(primary_handles)
    }

    /// Receives an ordered list of certificates and apply any application-specific logic.
    #[allow(clippy::type_complexity)]
    async fn post_process(mut rx_output: Receiver<(Result<ConsensusOutput, SubscriberError>, Vec<u8>)>, validator_state: Arc<ValidatorState>) {
        // TODO load the actual last seq num from db
        let mut last_seq_num = 0;
        loop {
            while let Some(message) = rx_output.recv().await {
                debug!("Received a finalized consensus transaction with analyze",);
                let (result, _serialized_txn) = message;
                match result {
                    Ok(consensus_output)=> {
                        if consensus_output.consensus_index > last_seq_num {
                            validator_state.validator_store.prune();
                            last_seq_num = consensus_output.consensus_index;
                        }
                    }
                    Err(_e) => () // TODO
                }

                // NOTE: Notify the user that its transaction has been processed.
            }
        }
    }

    async fn handle_transaction(
        consensus_adapter: Arc<Mutex<ConsensusAdapter>>,
        state: Arc<ValidatorState>,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<SignedTransaction>, tonic::Status> {
        debug!("Handling a new transaction with ValidatorService",);

        let transaction = request.into_inner();

        transaction
            .verify()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        // TODO change this to err flow
        // if !state
        //     .validator_store
        //     .check_seen_certificate_digest(transaction.get_transaction_payload().get_recent_certificate_digest())
        // {
        //     tonic::Status::internal("Invalid recent certificate digest");
        //     ();
        // }
        //
        // if state
        //     .validator_store
        //     .check_seen_transaction(transaction.get_transaction_payload())
        // {
        //     tonic::Status::internal("Duplicate transaction");
        //     ();
        // }

        let transaction_proto = TransactionProto {
            transaction: transaction.serialize().unwrap().into(),
        };

        let _result = consensus_adapter
            .lock()
            .await
            .consensus_client
            .submit_transaction(transaction_proto)
            .await
            .unwrap();

        state
            .handle_pre_consensus_transaction(&transaction)
            // .instrument(span)
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(transaction))
    }

    pub fn get_consensus_adapter(&self) -> &Arc<Mutex<ConsensusAdapter>> {
        &self.consensus_adapter
    }
}

/// Spawns a tonic grpc which parses incoming transactions and forwards them to the handle_transaction method of ValidatorService
#[async_trait]
impl ValidatorAPI for ValidatorService {
    async fn transaction(
        &self,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<SignedTransaction>, tonic::Status> {
        debug!("Handling a new transaction with a ValidatorService ValidatorAPI",);

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
        let validator_state = ValidatorState::new(public_key, secret, &genesis);
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
        let validator_state_0 = ValidatorState::new(public_key_0, secret_0, &genesis);
        let validator_state_1 = ValidatorState::new(public_key_1, secret_1, &genesis);

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
