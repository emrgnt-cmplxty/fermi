use super::state::ValidatorState;
use crate::config::node::NodeConfig;
use anyhow::anyhow;
use async_trait::async_trait;
use gdex_server::api::{ValidatorAPI, ValidatorAPIServer};
use gdex_types::{transaction::SignedTransaction, error::GDEXError};
use multiaddr::Multiaddr;
use prometheus::Registry;
use std::{io, sync::Arc, time::Duration};
use tokio::sync::mpsc::channel;
use tracing::{info, Instrument};
use gdex_types::crypto::KeypairTraits;
// use gdex_types::transaction::Hash;
use narwhal_crypto::Hash;

const MIN_BATCH_SIZE: u64 = 1000;
const MAX_DELAY_MILLIS: u64 = 5_000; // 5 sec

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

pub struct ValidatorServer {
    address: Multiaddr,
    pub state: Arc<ValidatorState>,
    min_batch_size: u64,
    max_delay: Duration,
}

impl ValidatorServer {
    pub fn new(address: Multiaddr, state: Arc<ValidatorState>, consensus_address: Multiaddr) -> Self {
        Self {
            address,
            state,
            // consensus_adapter,
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
            .add_service(ValidatorAPIServer::new(ValidatorService { state: self.state }))
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

pub struct ValidatorService {
    state: Arc<ValidatorState>,
}

impl ValidatorService {
    /// Spawn all the subsystems run by a Sui authority: a consensus node, a sui authority server,
    /// and a consensus listener bridging the consensus node and the sui authority.
    pub async fn new(config: &NodeConfig, state: Arc<ValidatorState>, prometheus_registry: &Registry) -> anyhow::Result<Self> {
        let (tx_consensus_to_sui, rx_consensus_to_sui) = channel(1_000);
        // let (tx_sui_to_consensus, rx_sui_to_consensus) = channel(1_000);

        // Spawn the consensus node of this authority.
        let consensus_config = config
            .consensus_config()
            .ok_or_else(|| anyhow!("Validator is missing consensus config"))?;
        let consensus_keypair = config.key_pair().copy();
        let consensus_name = consensus_keypair.public().clone();
        let consensus_store = narwhal_node::NodeStorage::reopen(consensus_config.db_path());
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

        Ok(Self { state })
    }

    async fn handle_transaction(
        state: Arc<ValidatorState>,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<SignedTransaction>, tonic::Status> {
        let transaction = request.into_inner();

        transaction
            .verify()
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        //TODO This is really really bad, we should have different types for signature-verified transactions
        // transaction.is_verified = true;

        let tx_digest = transaction.get_transaction_payload().digest();

        // Enable Trace Propagation across spans/processes using tx_digest
        // let span = tracing::debug_span!("process_tx", ?tx_digest, tx_kind = transaction.data.kind_as_str());

        let info = state
            .handle_transaction(&transaction)
            // .instrument(span)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(transaction))
    }
}

#[async_trait]
impl ValidatorAPI for ValidatorService {
    async fn transaction(
        &self,
        request: tonic::Request<SignedTransaction>,
    ) -> Result<tonic::Response<SignedTransaction>, tonic::Status> {
        let state = self.state.clone();

        // Spawns a task which handles the transaction. The task will unconditionally continue
        // processing in the event that the client connection is dropped.
        tokio::spawn(async move { Self::handle_transaction(state, request).await })
            .await
            .unwrap()
    }
}
