//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/genesis.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::builder::genesis_state::GenesisStateBuilder;
use anyhow::{Context, Result};
use gdex_controller::master::MasterController;
use gdex_types::{
    committee::{Committee, EpochId},
    error::GDEXResult,
    node::ValidatorInfo,
    serialization::{Base64, Encoding},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    convert::TryInto,
    {fs, path::Path},
};
use tracing::trace;

/// An object with the necessary state to initialize a new node at blockchain genesis
#[derive(Clone, Debug)]
pub struct ValidatorGenesisState {
    master_controller: MasterController,
    validator_set: Vec<ValidatorInfo>,
}

impl ValidatorGenesisState {
    pub fn new(master_controller: MasterController, validator_set: Vec<ValidatorInfo>) -> Self {
        ValidatorGenesisState {
            master_controller,
            validator_set,
        }
    }
    pub fn master_controller(&self) -> &MasterController {
        &self.master_controller
    }

    pub fn epoch(&self) -> EpochId {
        0
    }

    pub fn validator_set(&self) -> &[ValidatorInfo] {
        &self.validator_set
    }

    pub fn committee(&self) -> GDEXResult<Committee> {
        Committee::new(self.epoch(), ValidatorInfo::voting_rights(self.validator_set()))
    }

    pub fn narwhal_committee(&self) -> narwhal_config::SharedCommittee {
        let narwhal_committee = self
            .validator_set
            .iter()
            .map(|validator| {
                let name = validator.public_key().try_into().expect("Can't get narwhal public key");
                let primary = narwhal_config::PrimaryAddresses {
                    primary_to_primary: validator.narwhal_primary_to_primary.clone(),
                    worker_to_primary: validator.narwhal_worker_to_primary.clone(),
                };

                let mut worker_counter = 0;
                let workers = validator
                    .narwhal_primary_to_worker
                    .iter()
                    .zip(validator.narwhal_worker_to_worker.iter())
                    .map(|(primary_to_worker, worker_to_worker)| {
                        worker_counter += 1;
                        let transactions_address = validator
                            .narwhal_consensus_addresses
                            .get(worker_counter - 1)
                            .expect("Can't get worker consensus address")
                            .clone();
                        (
                            // TODO - find a less hacky way to iterate over the workers
                            (worker_counter - 1).try_into().unwrap(), // worker_id
                            narwhal_config::WorkerInfo {
                                primary_to_worker: primary_to_worker.clone(),
                                // TODO - change to triple zip
                                transactions: transactions_address,
                                worker_to_worker: worker_to_worker.clone(),
                            },
                        )
                    })
                    .collect();
                let validator = narwhal_config::Authority {
                    stake: validator.stake as narwhal_config::Stake, //TODO this should at least be the same size integer
                    primary,
                    workers,
                };

                (name, validator)
            })
            .collect();

        std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(narwhal_config::Committee {
            authorities: narwhal_committee,
            epoch: self.epoch() as narwhal_config::Epoch,
        }))
    }

    pub fn get_default_genesis() -> Self {
        GenesisStateBuilder::new().build()
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        trace!("Reading Genesis from {}", path.display());
        let bytes = fs::read(path).with_context(|| format!("Unable to load Genesis from {}", path.display()))?;
        Ok(bcs::from_bytes(&bytes)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        trace!("Writing Genesis to {}", path.display());
        let bytes = bcs::to_bytes(&self)?;
        fs::write(path, bytes).with_context(|| format!("Unable to save Genesis to {}", path.display()))?;
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("failed to serialize genesis")
    }
}

impl PartialEq for ValidatorGenesisState {
    fn eq(&self, other: &ValidatorGenesisState) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Serialize for ValidatorGenesisState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;

        #[derive(Serialize)]
        struct RawGenesis<'a> {
            master_controller: &'a MasterController,
            validator_set: &'a [ValidatorInfo],
        }

        let raw_genesis = RawGenesis {
            master_controller: &self.master_controller,
            validator_set: &self.validator_set,
        };

        let bytes = bcs::to_bytes(&raw_genesis).map_err(|e| Error::custom(e.to_string()))?;

        if serializer.is_human_readable() {
            let s = Base64::encode(&bytes);
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_bytes(&bytes)
        }
    }
}

impl<'de> Deserialize<'de> for ValidatorGenesisState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct RawGenesis {
            master_controller: MasterController,
            validator_set: Vec<ValidatorInfo>,
        }

        let bytes = if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Base64::decode(&s).map_err(|e| Error::custom(e.to_string()))?
        } else {
            let data: Vec<u8> = Vec::deserialize(deserializer)?;
            data
        };

        let raw_genesis: RawGenesis = bcs::from_bytes(&bytes).map_err(|e| Error::custom(e.to_string()))?;

        Ok(ValidatorGenesisState {
            master_controller: raw_genesis.master_controller,
            validator_set: raw_genesis.validator_set,
        })
    }
}
#[cfg(test)]
mod genesis_test {
    use super::*;
    use crate::{
        config::genesis::GenesisConfig,
        genesis_ceremony::{VALIDATOR_BALANCE, VALIDATOR_FUNDING_AMOUNT},
    };
    use gdex_types::{
        account::ValidatorKeyPair,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        utils,
    };

    #[test]
    fn roundtrip() {
        let genesis = GenesisStateBuilder::new().build();

        let s = serde_yaml::to_string(&genesis).unwrap();
        let from_s: ValidatorGenesisState = serde_yaml::from_str(&s).unwrap();
        assert_eq!(genesis, from_s);
    }

    #[test]
    fn ceremony() {
        let dir = tempfile::TempDir::new().unwrap();

        let _genesis_config = GenesisConfig::for_local_testing();

        let master_controller = MasterController::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        let key: ValidatorKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: key.public().into(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: vec![utils::new_network_address()],
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);
        builder.save(dir.path()).unwrap();
        GenesisStateBuilder::load(dir.path()).unwrap();
    }
}
