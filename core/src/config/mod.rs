//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/lib.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::validator::genesis_state::ValidatorGenesisState;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::trace;

pub mod consensus;
pub mod gateway;
pub mod genesis;
pub mod network;
pub mod node;
pub mod server;

pub const GDEX_NETWORK_CONFIG: &str = "network.yaml";
pub const GDEX_FULLNODE_CONFIG: &str = "fullnode.yaml";
pub const GDEX_CLIENT_CONFIG: &str = "client.yaml";
pub const GDEX_KEYSTORE_FILENAME: &str = "gdex.keystore";
pub const GDEX_GATEWAY_CONFIG: &str = "gateway.yaml";
pub const GDEX_GENESIS_FILENAME: &str = "genesis.blob";
pub const GDEX_DEV_NET_URL: &str = "https://gateway.devnet.sui.io:443";

pub const CONSENSUS_DB_NAME: &str = "consensus_db";
pub const GDEX_DB_NAME: &str = "gdex_db";
pub const DEFAULT_STAKE: u64 = crate::genesis_ceremony::VALIDATOR_FUNDING_AMOUNT;
pub const DEFAULT_BALANCE: u64 = crate::genesis_ceremony::VALIDATOR_BALANCE;

pub trait Config
where
    Self: DeserializeOwned + Serialize,
{
    fn persisted(self, path: &Path) -> PersistedConfig<Self> {
        PersistedConfig {
            inner: self,
            path: path.to_path_buf(),
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        trace!("Reading config from {}", path.display());
        let reader = fs::File::open(path).with_context(|| format!("Unable to load config from {}", path.display()))?;
        Ok(serde_yaml::from_reader(reader)?)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        trace!("Writing config to {}", path.display());
        let config = serde_yaml::to_string(&self)?;
        fs::write(path, config).with_context(|| format!("Unable to save config to {}", path.display()))?;
        Ok(())
    }
}

pub struct PersistedConfig<C> {
    inner: C,
    path: PathBuf,
}

impl<C> PersistedConfig<C>
where
    C: Config,
{
    pub fn read(path: &Path) -> Result<C, anyhow::Error> {
        Config::load(path)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.inner.save(&self.path)
    }

    pub fn into_inner(self) -> C {
        self.inner
    }
}

// This class is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/genesis.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum GenesisLocation {
    InPlace {
        genesis: ValidatorGenesisState,
    },
    File {
        #[serde(rename = "genesis-file-location")]
        genesis_file_location: PathBuf,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Genesis {
    #[serde(flatten)]
    location: GenesisLocation,

    #[serde(skip)]
    genesis: once_cell::sync::OnceCell<ValidatorGenesisState>,
}

impl Genesis {
    pub fn new(genesis: ValidatorGenesisState) -> Self {
        Self {
            location: GenesisLocation::InPlace { genesis },
            genesis: Default::default(),
        }
    }

    pub fn new_from_file<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            location: GenesisLocation::File {
                genesis_file_location: path.into(),
            },
            genesis: Default::default(),
        }
    }

    fn genesis(&self) -> Result<&ValidatorGenesisState> {
        match &self.location {
            GenesisLocation::InPlace { genesis } => Ok(genesis),
            GenesisLocation::File { genesis_file_location } => self
                .genesis
                .get_or_try_init(|| ValidatorGenesisState::load(&genesis_file_location)),
        }
    }
}

const GDEX_DIR: &str = ".gdex";
const GDEX_CONFIG_DIR: &str = "gdex_config";
pub fn gdex_config_dir() -> Result<PathBuf, anyhow::Error> {
    match std::env::var_os("GDEX_CONFIG_DIR") {
        Some(config_env) => Ok(config_env.into()),
        None => match dirs::home_dir() {
            Some(v) => Ok(v.join(GDEX_DIR).join(GDEX_CONFIG_DIR)),
            None => anyhow::bail!("Cannot obtain home directory path"),
        },
    }
    .and_then(|dir| {
        if !dir.exists() {
            std::fs::create_dir_all(dir.clone())?;
        }
        Ok(dir)
    })
}

/// Begin the testing suite for account
#[cfg(test)]
pub mod config {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize)]
    pub struct TestGenesisConfig {
        dummy: u64,
    }
    impl Config for TestGenesisConfig {}

    #[test]
    pub fn create_save_read_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = TestGenesisConfig { dummy: 1_000 };

        config.save(dir.path().join("test.conf")).unwrap();
        let config_load = TestGenesisConfig::load(dir.path().join("test.conf")).unwrap();
        assert!(config.dummy == config_load.dummy);
    }

    #[test]
    pub fn create_persisted_config_save_read() {
        let dir = tempfile::TempDir::new().unwrap();
        let persisted_config = PersistedConfig {
            path: dir.path().join("test.conf").into(),
            inner: TestGenesisConfig { dummy: 1_000 },
        };

        persisted_config.save().unwrap();

        let config_loaded: TestGenesisConfig = PersistedConfig::read(&dir.path().join("test.conf")).unwrap();

        assert!(persisted_config.into_inner().dummy == config_loaded.dummy);
    }
}
