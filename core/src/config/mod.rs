//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/lib.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::trace;

pub mod genesis;
pub mod genesis_ceremony;
pub mod genesis_config;

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
