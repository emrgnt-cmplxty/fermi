// fermi
use crate::builder::genesis_state::GenesisStateBuilder;
use fermi_controller::router::ControllerRouter;
use fermi_types::{
    committee::{Committee, EpochId},
    error::GDEXResult,
    node::ValidatorInfo,
    serialization::{Base64, Encoding},
};
// external
use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    convert::TryInto,
    {fs, path::Path},
};
use tracing::trace;

/// An object with the necessary state to initialize a new node at blockchain genesis
#[derive(Clone, Debug)]
pub struct ValidatorGenesisState {
    controller_router: ControllerRouter,
    validator_set: Vec<ValidatorInfo>,
}

impl ValidatorGenesisState {
    pub fn new(controller_router: ControllerRouter, validator_set: Vec<ValidatorInfo>) -> Self {
        ValidatorGenesisState {
            controller_router,
            validator_set,
        }
    }
    pub fn controller_router(&self) -> &ControllerRouter {
        &self.controller_router
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
                // Strong requirement here for narwhal and sui to be on the same version of fastcrypto
                // for AuthorityPublicBytes to cast to type alias PublicKey defined in narwhal to
                // construct narwhal Committee struct.
                let name = validator.public_key().try_into().expect("Can't get narwhal public key");
                let primary = narwhal_config::PrimaryAddresses {
                    primary_to_primary: validator.narwhal_primary_to_primary.clone(),
                    worker_to_primary: validator.narwhal_worker_to_primary.clone(),
                };
                let authority = narwhal_config::Authority {
                    stake: validator.stake as narwhal_config::Stake, //TODO this should at least be the same size integer
                    primary,
                };

                (name, authority)
            })
            .collect();
        std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(narwhal_config::Committee {
            authorities: narwhal_committee,
            epoch: self.epoch() as narwhal_config::Epoch,
        }))
    }

    pub fn narwhal_worker_cache(&self) -> narwhal_config::SharedWorkerCache {
        let workers = self
            .validator_set
            .iter()
            .map(|validator| {
                let name = validator.public_key().try_into().expect("Can't get narwhal public key");

                let mut worker_counter = 0;

                let workers = validator
                    .narwhal_primary_to_worker
                    .iter()
                    .zip(validator.narwhal_worker_to_worker.iter())
                    .zip(validator.narwhal_consensus_addresses.iter())
                    // map to triplet tuple
                    .map(|((primary_to_worker, worker_to_worker), consensus_address)| {
                        (primary_to_worker, worker_to_worker, consensus_address)
                    })
                    .map(|(primary_to_worker, worker_to_worker, consensus_address)| {
                        worker_counter += 1;
                        (
                            // unwrap is safe because we know worker_counter >= 0
                            (worker_counter - 1).try_into().unwrap(), // worker_id
                            narwhal_config::WorkerInfo {
                                primary_to_worker: primary_to_worker.clone(),
                                transactions: consensus_address.clone(),
                                worker_to_worker: worker_to_worker.clone(),
                            },
                        )
                    })
                    .collect();

                let worker_index = narwhal_config::WorkerIndex(workers);

                (name, worker_index)
            })
            .collect();
        narwhal_config::WorkerCache {
            workers,
            epoch: self.epoch() as narwhal_config::Epoch,
        }
        .into()
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
            controller_router: &'a ControllerRouter,
            validator_set: &'a [ValidatorInfo],
        }

        let raw_genesis = RawGenesis {
            controller_router: &self.controller_router,
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
            controller_router: ControllerRouter,
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

        raw_genesis.controller_router.initialize_controllers();

        Ok(ValidatorGenesisState {
            controller_router: raw_genesis.controller_router,
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
    use fermi_types::{
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

        let controller_router = ControllerRouter::default();
        controller_router.initialize_controllers();
        controller_router.initialize_controller_accounts();

        let key: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: key.public().into(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            balance: VALIDATOR_BALANCE,
            delegation: 0,
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: vec![utils::new_network_address()],
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(controller_router)
            .add_validator(validator);
        builder.save(dir.path()).unwrap();
        GenesisStateBuilder::load(dir.path()).unwrap();
    }
}