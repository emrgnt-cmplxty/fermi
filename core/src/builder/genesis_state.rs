//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/genesis.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::validator::genesis_state::ValidatorGenesisState;
use anyhow::{bail, Result};
use camino::Utf8Path;
use gdex_controller::master::MasterController;
use gdex_types::{account::ValidatorPubKeyBytes, node::ValidatorInfo, utils};
use std::{
    collections::BTreeMap,
    convert::TryInto,
    {fs, path::Path},
};
use tracing::trace;

/***fn create_genesis_objects() -> MasterController {
    MasterController::default()
}***/

const GENESIS_BUILDER_CONTROLLER_OUT: &str = "master_controller";
const GENESIS_BUILDER_COMMITTEE_DIR: &str = "committee";

/// Creates a builder object which facilitates the validator genesis construction
pub struct GenesisStateBuilder {
    pub master_controller: MasterController,
    pub validators: BTreeMap<ValidatorPubKeyBytes, ValidatorInfo>,
}

impl Default for GenesisStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GenesisStateBuilder {
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

    pub fn build(self) -> ValidatorGenesisState {
        let validators = self.validators.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
        let master_controller = self.master_controller; //create_genesis_objects();

        ValidatorGenesisState::new(master_controller, validators)
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
