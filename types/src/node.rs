//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
 
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use sui_types::base_types::SuiAddress;
use sui_types::committee::StakeUnit;
use sui_types::crypto::AuthorityPublicKeyBytes;

/// This class is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/node.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
/// Publicly known information about a validator
/// TODO read most of this from on-chain
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ValidatorInfo {
    pub name: String,
    pub public_key: AuthorityPublicKeyBytes,
    pub stake: StakeUnit,
    pub delegation: StakeUnit,
    pub network_address: Multiaddr,
    pub narwhal_primary_to_primary: Multiaddr,

    //TODO remove all of these as they shouldn't be needed to be encoded in genesis
    pub narwhal_worker_to_primary: Multiaddr,
    pub narwhal_primary_to_worker: Multiaddr,
    pub narwhal_worker_to_worker: Multiaddr,
    pub narwhal_consensus_address: Multiaddr,
}

impl ValidatorInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sui_address(&self) -> SuiAddress {
        (&self.public_key()).into()
    }

    pub fn public_key(&self) -> AuthorityPublicKeyBytes {
        self.public_key
    }

    pub fn stake(&self) -> StakeUnit {
        self.stake
    }

    pub fn delegation(&self) -> StakeUnit {
        self.delegation
    }

    pub fn network_address(&self) -> &Multiaddr {
        &self.network_address
    }

    pub fn voting_rights(validator_set: &[Self]) -> BTreeMap<AuthorityPublicKeyBytes, u64> {
        validator_set
            .iter()
            .map(|validator| (validator.public_key(), validator.stake() + validator.delegation()))
            .collect()
    }
}
