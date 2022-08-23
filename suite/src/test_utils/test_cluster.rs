// IMPORTS

// external
use std::{
    path::Path,
    path::PathBuf
};
use tempfile::TempDir;

// mysten

// gdex
use gdex_controller::master::MasterController;
use gdex_core::{
    genesis_ceremony::{
        VALIDATOR_FUNDING_AMOUNT,
        GENESIS_FILENAME
    },
    validator::{
        genesis_state::ValidatorGenesisState,
        spawner::ValidatorSpawner
    },
    
};
use gdex_types::{
    account::{
        ValidatorKeyPair,
        ValidatorPubKeyBytes
    },
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
    let validators_info = (0..validator_count)
        .map(|i| {
            let keypair: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
            let info = ValidatorInfo {
                name: format!("validator-{i}"),
                public_key: ValidatorPubKeyBytes::from(keypair.public()),
                stake: VALIDATOR_FUNDING_AMOUNT,
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
    ValidatorGenesisState::new(MasterController::default(), validators_info)
}

// INTERFACE

pub struct TestCluster {
    validator_count: usize,
    temp_working_dir: TempDir,
    genesis_state: ValidatorGenesisState,
    validator_spawners: Vec<ValidatorSpawner>
}

impl TestCluster {
    pub async fn new(
        validator_count: usize,
    ) -> Self {
        // get temp dirs
        let temp_working_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_working_dir.path().to_path_buf();

        // create genesis state
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
            genesis_state,
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
    
    pub fn get_genesis_state(
        &self
    ) -> ValidatorGenesisState {
        self.genesis_state.clone()
    }
    
    pub fn get_validator_spawner(
        &mut self,
        idx: usize
    ) -> &mut ValidatorSpawner {
        &mut self.validator_spawners[idx]
    }
}