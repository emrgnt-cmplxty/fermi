// IMPORTS

// local
use crate::{
    config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME, GRPC_DB_NAME, JSONRPC_DB_NAME},
    genesis_ceremony::GENESIS_FILENAME,
    json_rpc::spawner::JSONServiceSpawner,
    validator::{
        consensus_adapter::ConsensusAdapter, genesis_state::ValidatorGenesisState, metrics::ValidatorMetrics,
        post_processor::ValidatorPostProcessor, server::ValidatorServer, server::ValidatorService,
        state::ValidatorState,
    },
};

// gdex
use gdex_types::{node::ValidatorInfo, utils};

// external
use futures::future::join_all;
use multiaddr::Multiaddr;
use prometheus::Registry;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tracing::info;

// mysten
use narwhal_config::{Committee as ConsensusCommittee, Parameters as ConsensusParameters};
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use narwhal_node::metrics::start_prometheus_server;

// INTERFACE

// TODO - Cleanup the commentary in this file
// TODO - Create style guide for pub vs. getter functions

/// Can spawn a validator server handle at the internal address
/// the server handle contains a validator api (grpc) that exposes a validator service
pub struct ValidatorSpawner {
    /// Relative path where databases will be created and written to
    db_path: PathBuf,
    /// Relative path where validator keystore lives with convention {validator_name}.key
    key_path: PathBuf,

    /// Genesis state of the blockchain
    /// Note, it must contain corresponding information for this validator until more functionality is onboarded
    genesis_state: ValidatorGenesisState,
    /// Validator which is fetched from the genesis state according to initial name
    validator_info: ValidatorInfo,
    /// Address for communication to the validator GRPC server
    grpc_address: Multiaddr,
    /// Address for communication to the validator JSON RPC server
    // TODO - make this a multiaddr
    jsonrpc_address: Multiaddr,
    /// Address for communication to the metrics server
    metrics_address: Multiaddr,

    /// Begin objects initialized after calling spawn_validator_service

    /// Validator state passed to the instances spawned
    validator_state: Option<Arc<ValidatorState>>,
    /// Consensus adapter used to send transactions and fetch blocks
    consensus_adapter: Option<Arc<ConsensusAdapter>>,

    /// Begin objects initialized after calling spawn_validator_service

    /// Sender for the reconfiguration consensus service
    tx_reconfigure_consensus: Option<Sender<(ConsensusKeyPair, ConsensusCommittee)>>,
    /// Handle for the service related tasks
    service_handles: Option<Vec<JoinHandle<()>>>,
    /// Handle for the server related tasks
    server_handles: Option<Vec<JoinHandle<()>>>,
}

impl ValidatorSpawner {
    pub fn new(
        db_path: PathBuf,
        key_path: PathBuf,
        genesis_path: PathBuf,
        grpc_address: Multiaddr,
        jsonrpc_address: Multiaddr,
        metrics_address: Multiaddr,
        validator_name: String,
    ) -> Self {
        let genesis_state =
            ValidatorGenesisState::load(genesis_path.join(GENESIS_FILENAME)).expect("Could not open the genesis file");

        let validator_info = genesis_state
            .validator_set()
            .iter()
            .filter(|v| v.name == validator_name)
            .collect::<Vec<&ValidatorInfo>>()
            .pop()
            .expect("Could not locate validator {validator_name}")
            .clone();

        Self {
            db_path,
            key_path,
            genesis_state,
            validator_info,
            grpc_address,
            // TODO - make this configurable
            jsonrpc_address,
            metrics_address,
            validator_state: None,
            consensus_adapter: None,
            tx_reconfigure_consensus: None,
            service_handles: None,
            server_handles: None,
        }
    }

    // GETTERS
    pub fn get_grpc_address(&self) -> &Multiaddr {
        &self.grpc_address
    }

    pub fn get_jsonrpc_address(&self) -> &Multiaddr {
        &self.jsonrpc_address
    }

    pub fn get_validator_info(&self) -> &ValidatorInfo {
        &self.validator_info
    }

    pub fn get_validator_state(&self) -> Option<Arc<ValidatorState>> {
        if self.validator_state.is_some() {
            Some(Arc::clone(self.validator_state.as_ref().unwrap()))
        } else {
            None
        }
    }

    pub fn get_consensus_adapter(&self) -> Option<Arc<ConsensusAdapter>> {
        if self.consensus_adapter.is_some() {
            Some(Arc::clone(self.consensus_adapter.as_ref().unwrap()))
        } else {
            None
        }
    }

    pub fn get_genesis_state(&self) -> ValidatorGenesisState {
        self.genesis_state.clone()
    }

    fn is_validator_service_spawned(&self) -> bool {
        self.service_handles.is_some()
    }

    fn is_validator_server_spawned(&self) -> bool {
        self.server_handles.is_some()
    }

    // SETTERS
    pub fn halt_validator(&mut self) {
        self.validator_state.as_mut().unwrap().halt_validator();
    }

    pub fn unhalt_validator(&mut self) {
        self.validator_state.as_mut().unwrap().unhalt_validator();
    }

    fn set_validator_state(&mut self, validator_state: Arc<ValidatorState>) {
        self.validator_state = Some(validator_state)
    }

    fn set_consensus_adapter(&mut self, consensus_adapter: Arc<ConsensusAdapter>) {
        self.consensus_adapter = Some(consensus_adapter)
    }

    /// Internal helper function used to spawns the validator service
    /// note, this function will fail if called twice from the same spawner
    async fn spawn_validator_service(
        &mut self,
        rx_reconfigure_consensus: Receiver<(ConsensusKeyPair, ConsensusCommittee)>,
    ) {
        if self.is_validator_service_spawned() {
            panic!("The validator service has already been spawned");
        };

        // create config directory
        let consensus_addresses = self.validator_info.narwhal_consensus_addresses.clone();
        let pubilc_key = self.validator_info.public_key();

        // TODO - can we avoid consuming the private key twice in the network setup?
        // Note, this awkwardness is due to my inferred understanding of Arc pin.
        let key_file = &self.key_path;
        let consensus_db_path = self
            .db_path
            .join(format!("{}-{}", self.validator_info.name, CONSENSUS_DB_NAME));
        let grpc_db_path = self
            .db_path
            .join(format!("{}-{}", self.validator_info.name, GRPC_DB_NAME));

        info!(
            "Spawning a validator with the initial validator info = {:?}",
            self.validator_info
        );

        // Create a node config with this validators information
        let narwhal_config = ConsensusParameters { ..Default::default() };

        info!(
            "Spawning a validator with the input narwhal config = {:?}",
            narwhal_config
        );

        let consensus_config = ConsensusConfig {
            consensus_addresses,
            consensus_db_path: consensus_db_path.clone(),
            narwhal_config,
        };

        let key_pair = Arc::new(utils::read_keypair_from_file(&key_file).unwrap());
        let node_config = NodeConfig {
            key_pair,
            consensus_db_path,
            grpc_db_path: grpc_db_path.clone(),
            metrics_address: self.metrics_address.clone(),
            admin_interface_port: utils::get_available_port(),
            json_rpc_address: self.jsonrpc_address.clone(),
            websocket_address: Some(utils::available_local_socket_address()),
            consensus_config: Some(consensus_config),
            enable_event_processing: true,
            enable_gossip: true,
            enable_reconfig: false,
            genesis: Genesis::new(self.genesis_state.clone()),
        };

        let prom_address = node_config.metrics_address.clone();
        let prometheus_registry = Registry::new();
        info!("Starting Prometheus HTTP metrics endpoint at {}", prom_address);
        let prometheus_server_handle = vec![start_prometheus_server(prom_address, &prometheus_registry)];

        let metrics = Arc::new(ValidatorMetrics::new(&prometheus_registry));
        let validator_state = Arc::new(ValidatorState::new(
            pubilc_key,
            Arc::pin(utils::read_keypair_from_file(&key_file).unwrap()),
            &self.genesis_state,
            &grpc_db_path,
            metrics,
        ));

        // channel to communicate between narwhal + post process service
        let (tx_narwhal_to_post_process, rx_narwhal_to_post_process) = channel(1_000);

        let mut validator_handles: Vec<JoinHandle<()>> = Vec::new();

        // spawn the validator service, e.g. Narwhal consensus
        let narwhal_handles = ValidatorService::spawn_narwhal(
            &node_config,
            Arc::clone(&validator_state),
            &prometheus_registry,
            rx_reconfigure_consensus,
            tx_narwhal_to_post_process,
        )
        .unwrap();

        validator_handles.extend(narwhal_handles);
        validator_handles.extend(prometheus_server_handle);

        // spawn post process service
        let post_process_handles =
            ValidatorPostProcessor::spawn(rx_narwhal_to_post_process, Arc::clone(&validator_state)).unwrap();
        validator_handles.extend(post_process_handles);

        self.service_handles = Some(validator_handles);
        self.set_validator_state(validator_state);
    }

    /// Internal helper function used to spawns the validator server
    /// note, this function will fail if called twice from the same spawner
    async fn spawn_validator_server(&mut self) {
        if self.is_validator_server_spawned() {
            panic!("The validator server already been spawned");
        };

        let jsonrpc_db_path = self
            .db_path
            .join(format!("{}-{}", self.validator_info.name, JSONRPC_DB_NAME));

        let consensus_addresses = self.validator_info.narwhal_consensus_addresses.clone();
        let validator_server = ValidatorServer::new(
            self.grpc_address.clone(),
            // unwrapping is safe as validator state must have been created in spawn_validator_service
            Arc::clone(self.validator_state.as_ref().unwrap()),
            consensus_addresses,
            self.tx_reconfigure_consensus.as_ref().unwrap().clone(),
        );

        let validator_server_handle = validator_server.spawn().await.unwrap();
        self.set_consensus_adapter(validator_server_handle.get_adapter());

        let mut jsonrpc_spawner = JSONServiceSpawner::new(
            self.genesis_state.clone(),
            self.grpc_address.clone(),
            self.jsonrpc_address.clone(),
            jsonrpc_db_path.clone(),
        );

        let mut jsonrpc_handle = jsonrpc_spawner.spawn_jsonrpc_service().await.unwrap();
        jsonrpc_handle.push(validator_server_handle.get_handle());

        self.server_handles = Some(jsonrpc_handle);
    }

    pub async fn spawn_validator(&mut self) {
        // TODO assert this has not been called yet
        let (tx_reconfigure_consensus, rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);
        self.tx_reconfigure_consensus = Some(tx_reconfigure_consensus);
        self.spawn_validator_service(rx_reconfigure_consensus).await;
        self.spawn_validator_server().await;
        // Unwrap is safe since we have already launched the validator service
        self.validator_state.as_ref().unwrap().halt_validator();
    }

    pub async fn await_handles(&mut self) {
        join_all(self.service_handles.as_mut().unwrap()).await;
        join_all(self.server_handles.as_mut().unwrap()).await;
    }

    pub async fn stop(&mut self) {
        if let Some(handles) = self.service_handles.as_mut() {
            handles.iter().for_each(|h| h.abort());
        }
        if let Some(handles) = self.server_handles.as_mut() {
            handles.iter().for_each(|h| h.abort());
        }
        self.validator_state = None;
        self.server_handles = None;
        self.service_handles = None;
    }

    pub fn get_tx_reconfigure_consensus(&self) -> &Option<Sender<(ConsensusKeyPair, ConsensusCommittee)>> {
        &self.tx_reconfigure_consensus
    }
}
