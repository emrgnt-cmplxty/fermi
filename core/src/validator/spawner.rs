// IMPORTS

// local
use crate::{
    config::{
        consensus::ConsensusConfig,
        node::NodeConfig,
        Genesis,
        CONSENSUS_DB_NAME,
        GDEX_DB_NAME
    },
    genesis_ceremony::GENESIS_FILENAME,
    metrics::start_prometheus_server,
    validator::{
        genesis_state::ValidatorGenesisState,
        server::ValidatorServer,
        server::ValidatorService,
        state::ValidatorState,
    },
};

// gdex
use gdex_types::{
    node::ValidatorInfo,
    utils
};

// external
use multiaddr::Multiaddr;
use std::{
    path::PathBuf,
    sync::Arc
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::info;
use futures::future::join_all;

// mysten
use narwhal_config::{
    Committee as ConsensusCommittee,
    Parameters as ConsensusParameters
};
use narwhal_crypto::KeyPair as ConsensusKeyPair;



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
    /// Port for the validator to serve over
    validator_port: Multiaddr,

    /// Begin objects initialized after calling spawn_validator_service

    /// Validator state passed to the instances spawned
    validator_state: Option<Arc<ValidatorState>>,

    /// Begin objects initialized after calling spawn_validator_service

    /// Sender for the reconfiguration consensus service
    tx_reconfigure_consensus: Option<Sender<(ConsensusKeyPair, ConsensusCommittee)>>,
    /// Address for communication to the validator server
    validator_address: Option<Multiaddr>,
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
        validator_port: Multiaddr,
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
            .expect("Could not locate validator")
            .clone();
        Self {
            db_path,
            key_path,
            genesis_state,
            validator_port,
            validator_info,
            validator_state: None,
            validator_address: None,
            tx_reconfigure_consensus: None,
            service_handles: None,
            server_handles: None,
        }
    }

    pub fn get_validator_address(&self) -> &Option<Multiaddr> {
        &self.validator_address
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

    pub fn halt_validator(&mut self) {
        self.validator_state.as_mut().unwrap().halt_validator();
    }

    pub fn unhalt_validator(&mut self) {
        self.validator_state.as_mut().unwrap().unhalt_validator();
    }

    fn set_validator_state(&mut self, validator_state: Arc<ValidatorState>) {
        self.validator_state = Some(validator_state)
    }

    fn set_validator_address(&mut self, address: Multiaddr) {
        self.validator_address = Some(address)
    }

    fn is_validator_service_spawned(&self) -> bool {
        self.validator_state.is_some()
    }

    fn is_validator_server_spawned(&self) -> bool {
        self.validator_address.is_some()
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
        let network_address = self.validator_info.network_address.clone();
        let consensus_address = self.validator_info.narwhal_consensus_address.clone();
        let pubilc_key = self.validator_info.public_key();

        // TODO - can we avoid consuming the private key twice in the network setup?
        // Note, this awkwardness is due to my inferred understanding of Arc pin.
        let key_file = self.key_path.join(format!("{}.key", self.validator_info.name));
        let consensus_db_path = self
            .db_path
            .join(format!("{}-{}", self.validator_info.name, CONSENSUS_DB_NAME));
        let gdex_db_path = self
            .db_path
            .join(format!("{}-{}", self.validator_info.name, GDEX_DB_NAME));

        let key_pair = Arc::pin(utils::read_keypair_from_file(&key_file).unwrap());
        let validator_state = Arc::new(ValidatorState::new(
            pubilc_key,
            key_pair,
            &self.genesis_state,
            &gdex_db_path,
        ));

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
            consensus_address,
            consensus_db_path: consensus_db_path.clone(),
            narwhal_config,
        };
        let key_pair = Arc::new(utils::read_keypair_from_file(&key_file).unwrap());
        let node_config = NodeConfig {
            key_pair,
            consensus_db_path,
            gdex_db_path,
            network_address,
            metrics_address: utils::available_local_socket_address(),
            admin_interface_port: utils::get_available_port(),
            json_rpc_address: utils::available_local_socket_address(),
            websocket_address: Some(utils::available_local_socket_address()),
            consensus_config: Some(consensus_config),
            enable_event_processing: true,
            enable_gossip: true,
            enable_reconfig: false,
            genesis: Genesis::new(self.genesis_state.clone()),
        };
        let prometheus_registry = start_prometheus_server(node_config.metrics_address);

        // spawn the validator service, e.g. Narwhal consensus
        self.service_handles = Some(
            ValidatorService::spawn_narwhal(
                &node_config,
                Arc::clone(&validator_state),
                &prometheus_registry,
                rx_reconfigure_consensus,
            )
            .await
            .unwrap(),
        );
        self.set_validator_state(validator_state);
    }

    /// Internal helper function used to spawns the validator server
    /// note, this function will fail if called twice from the same spawner
    async fn spawn_validator_server(&mut self) {
        if self.is_validator_server_spawned() {
            panic!("The validator server already been spawned");
        };

        let consensus_address = self.validator_info.narwhal_consensus_address.clone();
        let validator_server = ValidatorServer::new(
            self.validator_port.clone(),
            // unwrapping is safe as validator state must have been created in spawn_validator_service
            Arc::clone(self.validator_state.as_ref().unwrap()),
            consensus_address,
            self.tx_reconfigure_consensus.as_ref().unwrap().clone(),
        );

        let validator_server_handle = validator_server.spawn().await.unwrap();
        self.set_validator_address(validator_server_handle.address().clone());

        self.server_handles = Some(vec![validator_server_handle.get_handle()]);
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
        self.validator_address = None;
        self.server_handles = None;
        self.service_handles = None;
    }

    pub fn get_tx_reconfigure_consensus(&self) -> &Option<Sender<(ConsensusKeyPair, ConsensusCommittee)>> {
        &self.tx_reconfigure_consensus
    }

    pub fn get_genesis_state(&self) -> ValidatorGenesisState {
        self.genesis_state.clone()
    }
}
