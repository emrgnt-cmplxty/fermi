use crate::{
    config::{consensus::ConsensusConfig, node::NodeConfig, Genesis, CONSENSUS_DB_NAME, GDEX_DB_NAME},
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
use narwhal_config::{Committee as ConsensusCommittee, Parameters as ConsensusParameters};
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
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
    /// Port for the validator to serve over
    validator_port: Multiaddr,

    /// Begin objects initialized after calling spawn_validator_service

    /// Validator state passed to the instances spawned
    validator_state: Option<Arc<ValidatorState>>,

    /// Begin objects initialized after calling spawn_validator_service

    /// Address for communication to the validator server
    validator_address: Option<Multiaddr>,
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
        }
    }

    pub fn get_validator_address(&self) -> &Option<Multiaddr> {
        &self.validator_address
    }

    pub fn get_validator_info(&self) -> &ValidatorInfo {
        &self.validator_info
    }

    pub fn get_validator_state(&mut self) -> &Option<Arc<ValidatorState>> {
        &self.validator_state
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
    ) -> Result<Vec<JoinHandle<()>>> {
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
        let spawned_service = ValidatorService::spawn_narwhal(
            &node_config,
            Arc::clone(&validator_state),
            &prometheus_registry,
            rx_reconfigure_consensus,
        )
        .await
        .unwrap();

        self.set_validator_state(validator_state);
        Ok(spawned_service)
    }

    /// Internal helper function used to spawns the validator server
    /// note, this function will fail if called twice from the same spawner
    pub async fn spawn_validator_server(
        &mut self,
        tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
    ) -> ValidatorServerHandle {
        if self.is_validator_server_spawned() {
            panic!("The validator server already been spawned");
        };

        let consensus_address = self.validator_info.narwhal_consensus_address.clone();
        let validator_server = ValidatorServer::new(
            self.validator_port.clone(),
            // unwrapping is safe as validator state must have been created in spawn_validator_service
            Arc::clone(self.validator_state.as_ref().unwrap()),
            consensus_address,
            tx_reconfigure_consensus,
        );

        let validator_handle = validator_server.spawn().await.unwrap();
        self.set_validator_address(validator_handle.address().clone());
        validator_handle
    }

    pub async fn spawn_validator(&mut self) -> Vec<JoinHandle<()>> {
        let (tx_reconfigure_consensus, rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);

        let mut join_handles = self.spawn_validator_service(rx_reconfigure_consensus).await.unwrap();
        let server_handle = self.spawn_validator_server(tx_reconfigure_consensus).await;
        join_handles.push(server_handle.get_handle());
        join_handles
    }

    #[cfg(test)]
    pub async fn spawn_validator_with_reconfigure(
        &mut self,
    ) -> (Vec<JoinHandle<()>>, Sender<(ConsensusKeyPair, ConsensusCommittee)>) {
        let (tx_reconfigure_consensus, rx_reconfigure_consensus) = tokio::sync::mpsc::channel(10);

        let mut join_handles = self.spawn_validator_service(rx_reconfigure_consensus).await.unwrap();
        let server_handle = self.spawn_validator_server(tx_reconfigure_consensus.clone()).await;
        join_handles.push(server_handle.get_handle());
        (join_handles, tx_reconfigure_consensus)
    }

    #[cfg(test)]
    pub fn get_genesis_state(&self) -> ValidatorGenesisState {
        return self.genesis_state.clone();
    }
}

#[cfg(test)]
pub mod suite_spawn_tests {
    use super::*;
    use crate::client;
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair},
        crypto::get_key_pair_from_rng,
        proto::{TransactionProto, TransactionsClient},
        transaction::{transaction_test_functions::generate_signed_test_transaction, SignedTransaction},
        utils,
    };
    use std::{io, path::Path};

    use tracing::info;
    use tracing_subscriber::FmtSubscriber;

    #[ignore]
    #[tokio::test]
    pub async fn spawn_node_and_reconfigure() {
        // let subscriber = FmtSubscriber::builder()
        //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        //     // will be written to stdout.
        //     .with_env_filter("info")
        //     .finish();
        // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        let dir = "../.proto";
        let path = Path::new(dir).to_path_buf();

        info!("Spawning validator");
        let address = utils::new_network_address();
        let mut spawner = ValidatorSpawner::new(
            /* db_path */ path.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ address.clone(),
            /* validator_name */ "validator-0".to_string(),
        );

        let handles = spawner.spawn_validator_with_reconfigure().await;

        info!("Sending 10 transactions");

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        let key_file = path.join(format!("{}.key", spawner.get_validator_info().name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let mut i = 0;
        while i < 10 {
            let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver, i);
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
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        info!("Reconfiguring validator");

        let consensus_committee = spawner.get_genesis_state().narwhal_committee().load().clone();
        let new_committee: narwhal_config::Committee = narwhal_config::Committee::clone(&consensus_committee);
        let new_committee: narwhal_config::Committee = narwhal_config::Committee {
            authorities: new_committee.authorities,
            epoch: 1,
        };

        let key = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let tx_reconfigure = handles.1;
        tx_reconfigure.send((key, new_committee)).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    #[tokio::test]
    #[ignore]
    pub async fn spawn_four_node_network() {
        let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        let dir = "../.proto";
        let temp_dir = tempfile::tempdir().unwrap().path().to_path_buf();
        let path = Path::new(dir).to_path_buf();

        info!("Spawning validator 0");
        let mut spawner_0 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-0".to_string(),
        );

        let _handler_0 = spawner_0.spawn_validator().await;

        info!("Spawning validator 1");
        let mut spawner_1 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-1".to_string(),
        );
        let _handler_1 = spawner_1.spawn_validator().await;

        info!("Spawning validator 2");
        let mut spawner_2 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-2".to_string(),
        );
        let _handler_2 = spawner_2.spawn_validator().await;

        info!("Spawning validator 3");
        let mut spawner_3 = ValidatorSpawner::new(
            /* db_path */ temp_dir.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ utils::new_network_address(),
            /* validator_name */ "validator-3".to_string(),
        );
        let _handler_3 = spawner_3.spawn_validator().await;

        info!("Sending transactions");
        let key_file = path.join(format!("{}.key", spawner_0.get_validator_info().name));
        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let address = spawner_0.get_validator_address().as_ref().unwrap().clone();
        info!("Connecting network client to address={:?}", address);

        let mut client =
            TransactionsClient::new(client::connect_lazy(&address).expect("Failed to connect to consensus"));

        let mut i = 1;
        let mut signed_transactions = Vec::new();
        let n_transactions_to_submit = 10;
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
        // sleep to allow the network to propagate the transactions
        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        let validator_store = &spawner_1
            .get_validator_state()
            .as_ref()
            .unwrap()
            .clone()
            .validator_store;

        // check that every transaction entered the cache
        for signed_transaction in signed_transactions.clone() {
            assert!(validator_store.contains_transaction(&signed_transaction.get_transaction_payload()));
        }

        let mut total = 0;
        let block_db = validator_store.block_store.iter(None).await;
        let mut block_db_iter = block_db.iter();

        while let Some(next_block) = block_db_iter.next() {
            let block = next_block.1;
            for serialized_transaction in &block.transactions {
                let signed_transaction_db = SignedTransaction::deserialize(serialized_transaction.clone()).unwrap();
                assert!(validator_store.contains_transaction(&signed_transaction_db.get_transaction_payload()));
                total += 1;
            }
            assert!(validator_store.contains_block_digest(&block.block_digest));
        }

        assert!(
            total as u64 == n_transactions_to_submit,
            "total transactions in db does not match total submitted"
        );
    }
}
