// IMPORTS

// external
use std::{
    path::Path,
    path::PathBuf
};
use tempfile::TempDir;

// mysten

// gdex
use gdex_controller::{
    bank::CREATED_ASSET_BALANCE,
    master::MasterController
};
use gdex_core::{
    genesis_ceremony::{
        VALIDATOR_FUNDING_AMOUNT,
        GENESIS_FILENAME,
        VALIDATOR_BALANCE
    },
    validator::{
        genesis_state::ValidatorGenesisState,
        spawner::ValidatorSpawner
    },
    
};
use gdex_types::{
    account::{
        ValidatorKeyPair,
        ValidatorPubKeyBytes,
        ValidatorPubKey
    },
    asset::PRIMARY_ASSET_ID,
    crypto::{
        get_key_pair_from_rng,
        KeypairTraits
    },
    node::ValidatorInfo,
    utils
};

// local

// HELPER FUNCTIONS

async fn create_genesis_state(
    dir: &Path,
    validator_count: usize
) -> ValidatorGenesisState {
    // initialize validator info
    let validators_info = (0..validator_count)
        .map(|i| {
            let keypair: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
            let info = ValidatorInfo {
                name: format!("validator-{i}"),
                public_key: ValidatorPubKeyBytes::from(keypair.public()),
                stake: VALIDATOR_FUNDING_AMOUNT,
                balance: VALIDATOR_BALANCE,
                delegation: 0,
                network_address: utils::new_network_address(),
                narwhal_primary_to_primary: utils::new_network_address(),
                narwhal_worker_to_primary: utils::new_network_address(),
                narwhal_primary_to_worker: utils::new_network_address(),
                narwhal_worker_to_worker: utils::new_network_address(),
                narwhal_consensus_address: utils::new_network_address(),
            };
            let key_file = dir.join(format!("{}.key", info.name));
            utils::write_keypair_to_file(&keypair, &key_file).unwrap();
            info
        })
        .collect::<Vec<_>>();
    
    let master_controller = MasterController::default();

    // create primary asset
    let validator_creator_pubkey = ValidatorPubKey::try_from(validators_info[0].public_key).unwrap();
    master_controller
        .bank_controller
        .lock()
        .unwrap()
        .create_asset(&validator_creator_pubkey)
        .unwrap();

    // fund validators
    let transfer_amount: u64 = CREATED_ASSET_BALANCE / (validator_count as u64);
    for validator_info in &validators_info {
        let validator_pubkey = ValidatorPubKey::try_from(validator_info.public_key).unwrap();
        master_controller
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
    
    ValidatorGenesisState::new(master_controller, validators_info)
}

// INTERFACE

pub struct TestCluster {
    validator_count: usize,
    temp_working_dir: TempDir,
    validator_spawners: Vec<ValidatorSpawner>
}

impl TestCluster {
    pub async fn new(
        validator_count: usize,
    ) -> Self {
        // get temp dirs
        let temp_working_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_working_dir.path().to_path_buf();

        // create and save genesis state
        let genesis_state = create_genesis_state(working_dir.as_path(), validator_count).await;
        let _save_result = genesis_state.save(working_dir.join(GENESIS_FILENAME));
    
        // create and spawn validators
        let mut validator_spawners: Vec<ValidatorSpawner> = Vec::new();
        for validator_info in genesis_state.validator_set() {
            let mut validator_spawner = ValidatorSpawner::new(
                working_dir.clone(), // db path
                working_dir.clone(), // key path
                working_dir.clone(), // genesis path
                validator_info.network_address.clone(),
                validator_info.name.clone()
            );
            validator_spawner.spawn_validator().await;
            validator_spawners.push(validator_spawner);
        }
        Self {
            validator_count,
            temp_working_dir,
            validator_spawners
        }
    }
    
    // GETTERS
    
    pub fn get_validator_count(
        &self
    ) -> usize {
        self.validator_count
    }
    
    pub fn get_working_dir(
        &self
    ) -> PathBuf {
        self.temp_working_dir.path().to_path_buf()
    }
    
    pub fn get_validator_spawner(
        &mut self,
        idx: usize
    ) -> &mut ValidatorSpawner {
        &mut self.validator_spawners[idx]
    }
}