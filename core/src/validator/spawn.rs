use super::server::ValidatorServerHandle;
use crate::{
    config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME},
    genesis_ceremony::GENESIS_FILENAME,
    metrics::start_prometheus_server,
    validator::{
        genesis_state::ValidatorGenesisState, server::ValidatorServer, server::ValidatorService, state::ValidatorState,
    },
};
use gdex_types::{node::ValidatorInfo, utils};
use multiaddr::Multiaddr;
use narwhal_config::Parameters as ConsensusParameters;
use std::{path::PathBuf, sync::Arc};

/// Can spawn a validator server handle at the internal address
/// the server handle contains a validator api (grpc) that exposes a validator service
pub struct ValidatorSpawner {
    path: PathBuf,
    genesis_state: ValidatorGenesisState,
    validator: ValidatorInfo,
    consensus_address: Option<Multiaddr>,
}

impl ValidatorSpawner {
    pub fn new(path: PathBuf, validator_name: String) -> Self {
        let genesis_state =
            ValidatorGenesisState::load(path.join(GENESIS_FILENAME)).expect("Could not open the genesis file");
        let validator = genesis_state
            .validator_set()
            .iter()
            .filter(|v| v.name == validator_name)
            .collect::<Vec<&ValidatorInfo>>()
            .pop()
            .expect("Could not locate validator")
            .clone();
        Self {
            path,
            genesis_state,
            validator,
            consensus_address: None,
        }
    }

    pub fn set_consensus_address(&mut self, address: Multiaddr) {
        self.consensus_address = Some(address)
    }

    async fn start_validator_service(&self) -> (Arc<ValidatorState>, ValidatorService) {
        // create config directory
        let network_address = self.validator.network_address.clone();
        let consensus_address = self.validator.narwhal_consensus_address.clone();
        let pubilc_key = self.validator.public_key();

        // TODO - can we avoid consuming the private key twice in the network setup?
        // Note, this awkwardness is due to my inferred understanding of Arc pin.
        let key_file = self.path.join(format!("{}.key", self.validator.name));
        let db_path = self.path.join(format!("{}-{}", self.validator.name, CONSENSUS_DB_NAME));

        let key_pair = Arc::pin(utils::read_keypair_from_file(&key_file).unwrap());
        let validator_state = Arc::new(ValidatorState::new(pubilc_key, key_pair, &self.genesis_state).await);

        println!("validator={:?}", self.validator);
        // Create a node config with this validators information
        let narwhal_config: ConsensusParameters = ConsensusParameters {
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
        println!("narwhal_config={:?}", narwhal_config);

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

        // spawn the validator service, e.g. Narwhal consensus
        let prometheus_registry = start_prometheus_server(node_config.metrics_address);
        let spawned_service = ValidatorService::new(&node_config, Arc::clone(&validator_state), &prometheus_registry)
            .await
            .unwrap();
        (validator_state, spawned_service)
    }

    pub async fn spawn(&mut self) -> ValidatorServerHandle {
        let (validator_state, validator_service) = self.start_validator_service().await;

        let new_addr = utils::new_network_address();
        let consensus_address = validator_service
            .consensus_adapter
            .lock()
            .await
            .consensus_address
            .to_owned();

        let validator_server = ValidatorServer::new(
            new_addr.clone(),
            Arc::clone(&validator_state),
            consensus_address,
            Some(
                self.genesis_state
                    .master_controller()
                    .consensus_controller
                    .min_batch_size,
            ),
            Some(
                self.genesis_state
                    .master_controller()
                    .consensus_controller
                    .max_batch_delay,
            ),
        );
        let validator_handle = validator_server.spawn().await.unwrap();
        self.set_consensus_address(validator_handle.address().clone());
        validator_handle
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

        // let mut validator_handles = Vec::new();
        info!("Spawning validator 0");
        let mut spawner_0 = ValidatorSpawner::new(
            /* path_dir */ path.clone(),
            /* validator_name */ "validator-0".to_string(),
        );

        let handler_0 = spawner_0.spawn().await;
        // validator_handles.push(spawner_0.spawn().await.handle);

        info!("Spawning validator 1");
        let mut spawner_1 = ValidatorSpawner::new(
            /* path_dir */ path.clone(),
            /* validator_name */ "validator-1".to_string(),
        );
        let handler_1 = spawner_1.spawn().await;

        info!("Spawning validator 2");
        let mut spawner_2 = ValidatorSpawner::new(
            /* path_dir */ path.clone(),
            /* validator_name */ "validator-2".to_string(),
        );
        let handler_2 = spawner_2.spawn().await;

        info!("Spawning validator 3");
        let mut spawner_3 = ValidatorSpawner::new(
            /* path_dir */ path.clone(),
            /* validator_name */ "validator-3".to_string(),
        );
        let handler_3 = spawner_3.spawn().await;

        info!("Sending transactions");
        let key_file = path.join(format!("{}.key", spawner_0.validator.name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let address = spawner_0.consensus_address.as_ref().unwrap().clone();
        println!("connecting network client to address={:?}", address);
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
        let _validator_handles = vec![handler_0.handle, handler_1.handle, handler_2.handle, handler_3.handle];
        // join_all(validator_handles).await;
    }
}
