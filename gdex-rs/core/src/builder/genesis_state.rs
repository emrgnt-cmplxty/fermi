// gdex
use crate::validator::genesis_state::ValidatorGenesisState;
use gdex_controller::router::ControllerRouter;
use gdex_types::utils::encode_bytes_hex;
use gdex_types::{account::ValidatorPubKeyBytes, node::ValidatorInfo};
// external
use anyhow::{bail, Result};
use camino::Utf8Path;
use std::{
    collections::BTreeMap,
    convert::TryInto,
    {fs, path::Path},
};
use tracing::trace;

const GENESIS_BUILDER_CONTROLLER_OUT: &str = "controller_router";
const GENESIS_BUILDER_COMMITTEE_DIR: &str = "committee";

/// Creates a builder object which facilitates the validator genesis construction
pub struct GenesisStateBuilder {
    pub controller_router: ControllerRouter,
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
            controller_router: Default::default(),
            validators: Default::default(),
        }
    }

    pub fn add_validator(mut self, validator: ValidatorInfo) -> Self {
        self.validators.insert(validator.public_key(), validator);
        self
    }

    pub fn set_master_controller(mut self, controller_router: ControllerRouter) -> Self {
        self.controller_router = controller_router;
        self
    }

    pub fn build(self) -> ValidatorGenesisState {
        let validators = self.validators.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
        let controller_router = self.controller_router; //create_genesis_objects();

        ValidatorGenesisState::new(controller_router, validators)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let path: &Utf8Path = path.try_into()?;
        trace!("Reading Genesis Builder from {}", path);

        if !path.is_dir() {
            bail!("path must be a directory");
        }

        // Load ControllerRouter
        let master_controller_bytes = fs::read(path.join(GENESIS_BUILDER_CONTROLLER_OUT))?;
        let controller_router: ControllerRouter = serde_yaml::from_slice(&master_controller_bytes)?;

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
            controller_router,
            validators: committee,
        })
    }

    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        trace!("Writing Genesis Builder to {}", path.display());

        std::fs::create_dir_all(path)?;

        // Write Objects
        let master_controller_dir = path.join(GENESIS_BUILDER_CONTROLLER_OUT);
        let master_controller_bytes = serde_yaml::to_vec(&self.controller_router)?;
        fs::write(master_controller_dir, master_controller_bytes)?;

        // Write validator infos
        let committee_dir = path.join(GENESIS_BUILDER_COMMITTEE_DIR);
        std::fs::create_dir_all(&committee_dir)?;

        for (_pubkey, validator) in self.validators {
            let validator_info_bytes = serde_yaml::to_vec(&validator)?;
            let hex_name = encode_bytes_hex(validator.public_key());
            fs::write(committee_dir.join(hex_name.as_str()), validator_info_bytes)?;
        }

        Ok(())
    }
}
