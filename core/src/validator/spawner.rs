use crate::{
    config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME},
    genesis_ceremony::GENESIS_FILENAME,
    metrics::start_prometheus_server,
    validator::{
        genesis_state::ValidatorGenesisState, server::ValidatorServer, server::ValidatorServerHandle,
        server::ValidatorService, state::ValidatorState,
    },
};
use anyhow::Result;
use gdex_types::{node::ValidatorInfo, utils};
use multiaddr::Multiaddr;
use narwhal_config::Parameters as ConsensusParameters;
use std::{path::PathBuf, sync::Arc};
use tokio::task::JoinHandle;
use tracing::info;
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

    /// Begin objects initialized after calling spawn_validator_service

    /// Validator state passed to the instances spawned
    validator_state: Option<Arc<ValidatorState>>,

    /// Begin objects initialized after calling spawn_validator_service

    /// Address for communication to the validator server
    validator_address: Option<Multiaddr>,
}

impl ValidatorSpawner {
    pub fn new(db_path: PathBuf, key_path: PathBuf, genesis_path: PathBuf, validator_name: String) -> Self {
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
            validator_info,
            validator_state: None,
            validator_address: None,
        }
    }

    pub fn get_validator_address(&self) -> &Option<Multiaddr> {
        &self.validator_address
    }

    pub fn get_validator_info(&self) -> &ValidatorInfo {
        &self.validator_info
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

    async fn spawn_validator_service(&mut self) -> Result<Vec<JoinHandle<()>>> {
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
        let db_path = self
            .db_path
            .join(format!("{}-{}", self.validator_info.name, CONSENSUS_DB_NAME));

        let key_pair = Arc::pin(utils::read_keypair_from_file(&key_file).unwrap());
        let validator_state = Arc::new(ValidatorState::new(pubilc_key, key_pair, &self.genesis_state));

        info!(
            "Spawning a validator with the initial validator info = {:?}",
            self.validator_info
        );

        // Create a node config with this validators information
        let narwhal_config = ConsensusParameters {
            batch_size: self
                .genesis_state
                .master_controller()
                .consensus_controller
                .min_batch_size,
            max_batch_delay: self
                .genesis_state
                .master_controller()
                .consensus_controller
                .max_batch_delay,
            ..Default::default()
        };

        info!(
            "Spawning a validator with the input narwhal config = {:?}",
            narwhal_config
        );

        let consensus_config = ConsensusConfig {
            consensus_address,
            consensus_db_path: db_path.clone(),
            narwhal_config,
        };
        let key_pair = Arc::new(utils::read_keypair_from_file(&key_file).unwrap());
        let node_config = NodeConfig {
            key_pair,
            db_path,
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
        let spawned_service =
            ValidatorService::spawn_narwhal(&node_config, Arc::clone(&validator_state), &prometheus_registry)
                .await
                .unwrap();

        self.set_validator_state(validator_state);
        Ok(spawned_service)
    }

    pub async fn spawn_validator_server(&mut self) -> ValidatorServerHandle {
        if self.is_validator_server_spawned() {
            panic!("The validator server already been spawned");
        };

        let new_addr = utils::new_network_address();
        let consensus_address = self.validator_info.narwhal_consensus_address.clone();

        let validator_server = ValidatorServer::new(
            new_addr.clone(),
            // unwrapping is safe as validator state must have been created in spawn_validator_service
            Arc::clone(self.validator_state.as_ref().unwrap()),
            consensus_address,
        );
        let validator_handle = validator_server.spawn().await.unwrap();
        self.set_validator_address(validator_handle.address().clone());
        validator_handle
    }

    pub async fn spawn_validator(&mut self) -> (Multiaddr, Vec<JoinHandle<()>>) {
        let mut join_handles = self.spawn_validator_service().await.unwrap();
        let server_handle = self.spawn_validator_server().await;
        let address = server_handle.address().to_owned();
        join_handles.push(server_handle.get_handle());
        (address, join_handles)
    }
}

#[cfg(test)]
pub mod suite_spawn_tests {
    use super::*;
    use crate::client::{ClientAPI, NetworkValidatorClient};
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair},
        transaction::transaction_test_functions::generate_signed_test_transaction,
        utils,
    };
    use std::{io, path::Path};
    use tracing::info;
    use tracing_subscriber::FmtSubscriber;

    #[tokio::test]
    #[ignore]
    pub async fn test() {
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_env_filter("gdex_core=debug, gdex_suite=debug")
            // .with_max_level(Level::DEBUG)
            // completes the builder.
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        let dir = "../.proto";
        let path = Path::new(dir).to_path_buf();

        info!("Spawning validator 0");
        let mut spawner_0 = ValidatorSpawner::new(
            /* db_path */ path.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_name */ "validator-0".to_string(),
        );

        let handler_0 = spawner_0.spawn_validator().await;

        info!("Spawning validator 1");
        let mut spawner_1 = ValidatorSpawner::new(
            /* db_path */ path.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_name */ "validator-1".to_string(),
        );
        let _handler_1 = spawner_1.spawn_validator().await;

        info!("Spawning validator 2");
        let mut spawner_2 = ValidatorSpawner::new(
            /* db_path */ path.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_name */ "validator-2".to_string(),
        );
        let _handler_2 = spawner_2.spawn_validator().await;

        info!("Spawning validator 3");
        let mut spawner_3 = ValidatorSpawner::new(
            /* db_path */ path.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_name */ "validator-3".to_string(),
        );
        let _handler_3 = spawner_3.spawn_validator().await;

        info!("Sending transactions");
        let key_file = path.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        info!("Connecting network client to address={:?}", address);
        let client = NetworkValidatorClient::connect_lazy(&address).unwrap();
        let mut i = 0;
        while i < 1_000 {
            let _resp1 = client
                .handle_transaction(signed_transaction.clone())
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
            i += 1;
        }
    }
}
