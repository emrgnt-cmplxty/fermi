// mysten
use narwhal_config::Parameters as ConsensusParameters;
// external
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configures the local validators consensus participation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConsensusConfig {
    /// The address for communicating with other nodes in the network
    pub consensus_addresses: Vec<Multiaddr>,
    /// Path to the consensus database
    pub consensus_db_path: PathBuf,
    /// Narwhal consensus parameters
    pub narwhal_config: ConsensusParameters,
}

impl ConsensusConfig {
    pub fn addresses(&self) -> &Vec<Multiaddr> {
        &self.consensus_addresses
    }

    pub fn db_path(&self) -> &Path {
        &self.consensus_db_path
    }

    pub fn narwhal_config(&self) -> &ConsensusParameters {
        &self.narwhal_config
    }
}

#[cfg(test)]
mod consensus_tests {
    use super::*;
    use gdex_types::utils;
    #[test]
    pub fn config() {
        let new_address = utils::new_network_address();
        let new_config = ConsensusConfig {
            consensus_addresses: vec![new_address],
            consensus_db_path: PathBuf::from("test.conf"),
            narwhal_config: Default::default(),
        };
        // quick checks on newly created config
        assert!(!new_config.addresses().is_empty());
        assert!(new_config.db_path() == PathBuf::from("test.conf"));
        assert!(new_config.narwhal_config().batch_size > 0);
    }
}
