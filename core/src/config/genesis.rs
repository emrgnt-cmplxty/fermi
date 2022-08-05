//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/genesis.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc

use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use gdex_proc::master::MasterController;
use gdex_types::{
    account::AuthorityPubKeyBytes,
    committee::{Committee, EpochId},
    error::GDEXResult,
    node::ValidatorInfo,
    serialization::{Base64, Encoding},
    utils,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::{fs, path::Path};
use tracing::trace;

#[derive(Clone, Debug)]
pub struct Genesis {
    master_controller: MasterController,
    validator_set: Vec<ValidatorInfo>,
}

impl Genesis {
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
                let workers = [(
                    0, // worker_id
                    narwhal_config::WorkerAddresses {
                        primary_to_worker: validator.narwhal_primary_to_worker.clone(),
                        transactions: validator.narwhal_consensus_address.clone(),
                        worker_to_worker: validator.narwhal_worker_to_worker.clone(),
                    },
                )]
                .into_iter()
                .collect();
                let authority = narwhal_config::Authority {
                    stake: validator.stake as narwhal_config::Stake, //TODO this should at least be the same size integer
                    primary,
                    workers,
                };

                (name, authority)
            })
            .collect();
        std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(narwhal_config::Committee {
            authorities: narwhal_committee,
            epoch: self.epoch() as narwhal_config::Epoch,
        }))
    }

    pub fn get_default_genesis() -> Self {
        Builder::new().build()
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

impl PartialEq for Genesis {
    fn eq(&self, other: &Genesis) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

impl Serialize for Genesis {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;

        #[derive(Serialize)]
        struct RawGeneis<'a> {
            master_controller: &'a MasterController,
            validator_set: &'a [ValidatorInfo],
        }

        let raw_genesis = RawGeneis {
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

impl<'de> Deserialize<'de> for Genesis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct RawGeneis {
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

        let raw_genesis: RawGeneis = bcs::from_bytes(&bytes).map_err(|e| Error::custom(e.to_string()))?;

        Ok(Genesis {
            master_controller: raw_genesis.master_controller,
            validator_set: raw_genesis.validator_set,
        })
    }
}

pub struct Builder {
    pub master_controller: MasterController,
    pub validators: BTreeMap<AuthorityPubKeyBytes, ValidatorInfo>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            master_controller: Default::default(),
            validators: Default::default(),
        }
    }

    pub fn add_validator(mut self, validator: ValidatorInfo) -> Self {
        self.validators.insert(validator.public_key(), validator);
        self
    }

    pub fn set_master_controller(mut self, master_controller: MasterController) -> Self {
        self.master_controller = master_controller;
        self
    }

    pub fn build(self) -> Genesis {
        let validators = self.validators.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
        let master_controller = create_genesis_objects();

        Genesis {
            master_controller,
            validator_set: validators,
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let path: &Utf8Path = path.try_into()?;
        trace!("Reading Genesis Builder from {}", path);

        if !path.is_dir() {
            bail!("path must be a directory");
        }

        // Load MasterController
        let master_controller_bytes = fs::read(path.join(GENESIS_BUILDER_CONTROLLER_OUT))?;
        let master_controller: MasterController = serde_yaml::from_slice(&master_controller_bytes)?;

        // Load validator infos
        let mut committee = BTreeMap::new();
        for entry in path.join(GENESIS_BUILDER_COMMITTEE_DIR).read_dir_utf8()? {
            let entry = entry?;
            if entry.file_name().starts_with('.') {
                continue;
            }

            let path = entry.path();
            let validator_info_bytes = fs::read(path)?;
            let validator_info: ValidatorInfo = serde_yaml::from_slice(&validator_info_bytes)?;
            committee.insert(validator_info.public_key(), validator_info);
        }

        Ok(Self {
            master_controller,
            validators: committee,
        })
    }

    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        trace!("Writing Genesis Builder to {}", path.display());

        std::fs::create_dir_all(path)?;

        // Write Objects
        let master_controller_dir = path.join(GENESIS_BUILDER_CONTROLLER_OUT);
        let master_controller_bytes = serde_yaml::to_vec(&self.master_controller)?;
        fs::write(master_controller_dir, master_controller_bytes)?;

        // Write validator infos
        let committee_dir = path.join(GENESIS_BUILDER_COMMITTEE_DIR);
        std::fs::create_dir_all(&committee_dir)?;

        for (_pubkey, validator) in self.validators {
            let validator_info_bytes = serde_yaml::to_vec(&validator)?;
            let hex_name = utils::encode_bytes_hex(&validator.public_key());
            fs::write(committee_dir.join(hex_name), validator_info_bytes)?;
        }

        Ok(())
    }
}

fn create_genesis_objects() -> MasterController {
    MasterController::default()
}

const GENESIS_BUILDER_CONTROLLER_OUT: &str = "master_controller";
const GENESIS_BUILDER_COMMITTEE_DIR: &str = "committee";

#[cfg(test)]
mod test {
    use super::super::genesis_config::GenesisConfig;
    use super::*;
    use gdex_types::{
        account::AuthorityKeyPair,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        utils,
    };

    #[test]
    fn roundtrip() {
        let genesis = Builder::new().build();

        let s = serde_yaml::to_string(&genesis).unwrap();
        let from_s: Genesis = serde_yaml::from_str(&s).unwrap();
        assert_eq!(genesis, from_s);
    }

    #[test]
    fn ceremony() {
        let dir = tempfile::TempDir::new().unwrap();

        let _genesis_config = GenesisConfig::for_local_testing();

        let master_controller = MasterController::default();

        let key: AuthorityKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: key.public().into(),
            stake: 1,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let builder = Builder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);
        builder.save(dir.path()).unwrap();
        Builder::load(dir.path()).unwrap();
    }
}
