// fermi
use fermi_controller::{
    bank::{
        controller::CREATED_ASSET_BALANCE, proto::bank_controller_test_functions::generate_signed_test_transaction,
    },
    router::ControllerRouter,
};
use fermi_core::{
    client,
    genesis_ceremony::{GENESIS_FILENAME, VALIDATOR_BALANCE, VALIDATOR_FUNDING_AMOUNT},
    validator::{genesis_state::ValidatorGenesisState, spawner::ValidatorSpawner},
};
use fermi_types::{
    account::{account_test_functions::generate_keypair_vec, ValidatorKeyPair, ValidatorPubKey, ValidatorPubKeyBytes},
    asset::PRIMARY_ASSET_ID,
    crypto::{get_key_pair_from_rng, KeypairTraits},
    node::ValidatorInfo,
    proto::ValidatorGrpcClient,
    transaction::SignedTransaction,
    utils,
};
// external
use std::{io, path::Path, path::PathBuf};
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use tracing::info;

// HELPER FUNCTIONS

async fn create_genesis_state(dir: &Path, validator_count: usize) -> ValidatorGenesisState {
    // initialize validator info
    let validators_info = (0..validator_count)
        .map(|i| {
            let keypair: ValidatorKeyPair =
                get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
            let info = ValidatorInfo {
                name: format!("validator-{i}"),
                public_key: ValidatorPubKeyBytes::from(keypair.public()),
                stake: VALIDATOR_FUNDING_AMOUNT,
                balance: VALIDATOR_BALANCE,
                delegation: 0,
                narwhal_primary_to_primary: utils::new_network_address(),
                narwhal_worker_to_primary: utils::new_network_address(),
                narwhal_primary_to_worker: vec![utils::new_network_address()],
                narwhal_worker_to_worker: vec![utils::new_network_address()],
                narwhal_consensus_addresses: vec![utils::new_network_address()],
            };
            let key_file = dir.join(format!("{}.key", info.name));
            utils::write_keypair_to_file(&keypair, &key_file).unwrap();
            info
        })
        .collect::<Vec<_>>();

    let controller_router = ControllerRouter::default();
    controller_router.initialize_controllers();
    controller_router.initialize_controller_accounts();

    // create primary asset
    let validator_creator_pubkey = ValidatorPubKey::try_from(validators_info[0].public_key).unwrap();
    controller_router
        .bank_controller
        .lock()
        .unwrap()
        .create_asset(&validator_creator_pubkey)
        .unwrap();

    // fund validators
    let transfer_amount: u64 = CREATED_ASSET_BALANCE / (validator_count as u64);
    for validator_info in &validators_info {
        let validator_pubkey = ValidatorPubKey::try_from(validator_info.public_key).unwrap();
        controller_router
            .bank_controller
            .lock()
            .unwrap()
            .transfer(
                &validator_creator_pubkey,
                &validator_pubkey,
                PRIMARY_ASSET_ID,
                transfer_amount,
            )
            .unwrap();
    }

    ValidatorGenesisState::new(controller_router, validators_info)
}

// INTERFACE
pub struct TestCluster {
    validator_count: usize,
    temp_working_dir: TempDir,
    validator_spawners: Vec<ValidatorSpawner>,
}

impl TestCluster {
    pub async fn spawn(validator_count: usize, max_spawn: Option<usize>) -> Self {
        // get temp dirs
        let temp_working_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_working_dir.path().to_path_buf();

        // create and save genesis state
        let genesis_state = create_genesis_state(working_dir.as_path(), validator_count).await;
        let _save_result = genesis_state.save(working_dir.join(GENESIS_FILENAME));

        // create and spawn validators
        let mut validator_spawners = Vec::new();

        let mut validator_counter = 0;
        for validator_info in genesis_state.validator_set() {
            validator_counter += 1;
            let key_file = format!("{}.key", validator_info.name);
            let mut validator_spawner = ValidatorSpawner::new(
                working_dir.clone(),
                PathBuf::from(working_dir.to_str().unwrap().to_owned() + "/" + key_file.as_str()),
                working_dir.clone(),
                utils::new_network_address(),
                utils::new_network_address(),
                utils::new_network_address(),
                validator_info.name.clone(),
            );

            if validator_counter <= max_spawn.unwrap_or(validator_count) {
                info!("Spawning validator {}", validator_counter);
                validator_spawner.spawn_validator().await;
                validator_spawner.get_validator_state().unwrap().unhalt_validator();
            }

            validator_spawners.push(validator_spawner);
        }

        // sleep
        sleep(Duration::from_secs(1)).await;

        Self {
            validator_count,
            temp_working_dir,
            validator_spawners,
        }
    }

    // GETTERS

    pub fn get_validator_count(&self) -> usize {
        self.validator_count
    }

    pub fn get_working_dir(&self) -> PathBuf {
        self.temp_working_dir.path().to_path_buf()
    }

    // TODO - we need a non-mut instance of this function for testing.
    pub fn get_validator_spawner(&mut self, index: usize) -> &mut ValidatorSpawner {
        &mut self.validator_spawners[index]
    }

    pub async fn stop(&mut self, index: usize) {
        let spawner = self.get_validator_spawner(index);
        spawner.stop().await;
    }

    pub async fn start(&mut self, index: usize) {
        let spawner = self.get_validator_spawner(index);
        // start the validator back up
        spawner.spawn_validator().await;
        spawner.get_validator_state().unwrap().unhalt_validator();
    }

    pub fn get_validator_client(&mut self, index: usize) -> ValidatorGrpcClient<Channel> {
        let receiver = self.get_validator_spawner(index);
        let receiver_address = receiver.get_grpc_address().clone();
        ValidatorGrpcClient::new(client::connect_lazy(&receiver_address).expect("Failed to connect to consensus"))
    }

    pub async fn send_single_transaction(&mut self, signed_transaction: SignedTransaction) {
        let receiver = self.get_validator_spawner(0);
        let receiver_address = receiver.get_grpc_address().clone();

        let mut client =
            ValidatorGrpcClient::new(client::connect_lazy(&receiver_address).expect("Failed to connect to consensus"));
        client
            .submit_transaction(signed_transaction)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .unwrap();
    }

    pub async fn send_transactions(
        &mut self,
        sending_validator: usize,
        receiving_validator: usize,
        n_transactions: u64,
    ) -> (ValidatorKeyPair, ValidatorKeyPair, Vec<SignedTransaction>) {
        let working_dir = self.get_working_dir();
        let sender = self.get_validator_spawner(sending_validator);
        let key_file = working_dir.join(format!("{}.key", sender.get_validator_info().name));

        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let receiver = self.get_validator_spawner(receiving_validator);
        let receiver_address = receiver.get_grpc_address().clone();

        let mut client =
            ValidatorGrpcClient::new(client::connect_lazy(&receiver_address).expect("Failed to connect to consensus"));

        let mut signed_transactions = Vec::new();
        let mut i = 1;
        while i < n_transactions + 1 {
            let amount = i;
            let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver, amount);
            signed_transactions.push(signed_transaction.clone());
            let _resp1 = client
                .submit_transaction(signed_transaction)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .unwrap();
            i += 1;
        }
        (kp_sender, kp_receiver, signed_transactions)
    }

    pub async fn send_transactions_async(
        &mut self,
        sending_validator: usize,
        receiving_validator: usize,
        n_transactions: u64,
        fixed_amount: Option<u64>,
    ) -> JoinHandle<()> {
        let working_dir = self.get_working_dir();
        let sender = self.get_validator_spawner(sending_validator);
        let key_file = working_dir.join(format!("{}.key", sender.get_validator_info().name));

        let kp_sender: ValidatorKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let receiver = self.get_validator_spawner(receiving_validator);
        let receiver_address = receiver.get_grpc_address().clone();

        let mut client =
            ValidatorGrpcClient::new(client::connect_lazy(&receiver_address).expect("Failed to connect to consensus"));

        let mut signed_transactions = Vec::new();
        let mut i = 1;
        tokio::spawn(async move {
            while i < n_transactions + 1 {
                let amount = fixed_amount.unwrap_or(i);
                let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver, amount);
                signed_transactions.push(signed_transaction.clone());
                let _resp1 = client
                    .submit_transaction(signed_transaction)
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    .unwrap();
                i += 1;
            }
        })
    }
}