//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/node.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use multiaddr::Multiaddr;
use narwhal_config::Parameters as ConsensusParameters;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configures the local validators consensus participation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConsensusConfig {
    /// The address for communicating with other nodes in the network
    pub consensus_address: Multiaddr,
    /// Path to the consensus database
    pub consensus_db_path: PathBuf,
    /// Narwhal consensus parameters
    pub narwhal_config: ConsensusParameters,
}

impl ConsensusConfig {
    pub fn address(&self) -> &Multiaddr {
        &self.consensus_address
    }

    pub fn db_path(&self) -> &Path {
        &self.consensus_db_path
    }

    pub fn narwhal_config(&self) -> &ConsensusParameters {
        &self.narwhal_config
    }
}
