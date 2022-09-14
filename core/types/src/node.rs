//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::account::ValidatorPubKeyBytes;
use crate::crypto::GDEXAddress;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use sui_types::committee::StakeUnit;

/// This class is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/node.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
/// Publicly known information about a validator
/// TODO read most of this from on-chain
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ValidatorInfo {
    pub name: String,
    pub public_key: ValidatorPubKeyBytes,
    pub stake: StakeUnit,
    pub balance: u64,
    pub delegation: StakeUnit,
    pub narwhal_primary_to_primary: Multiaddr,
    pub narwhal_worker_to_primary: Multiaddr,
    pub narwhal_primary_to_worker: Vec<Multiaddr>,
    pub narwhal_worker_to_worker: Vec<Multiaddr>,
    pub narwhal_consensus_addresses: Vec<Multiaddr>,
}

impl ValidatorInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn gdex_address(&self) -> GDEXAddress {
        (&self.public_key()).into()
    }

    pub fn public_key(&self) -> ValidatorPubKeyBytes {
        self.public_key
    }

    pub fn stake(&self) -> StakeUnit {
        self.stake
    }

    pub fn delegation(&self) -> StakeUnit {
        self.delegation
    }

    pub fn voting_rights(validator_set: &[Self]) -> BTreeMap<ValidatorPubKeyBytes, u64> {
        validator_set
            .iter()
            .map(|validator| (validator.public_key(), validator.stake() + validator.delegation()))
            .collect()
    }
}

/// Begin the testing suite for transactions
#[cfg(test)]
pub mod node_tests {
    use super::*;

    use crate::account::account_test_functions::generate_keypair_vec;
    use crate::crypto::KeypairTraits;
    use crate::utils;

    #[test]
    pub fn validator_info() {
        let kp = generate_keypair_vec([0; 32]).pop().unwrap();

        let name = "0";
        let stake = 1;
        let balance = 2;
        let delegation = 0;
        let validator = ValidatorInfo {
            name: name.into(),
            public_key: kp.public().into(),
            stake,
            balance,
            delegation,
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: vec![utils::new_network_address()],
            narwhal_worker_to_worker: vec![utils::new_network_address()],
            narwhal_consensus_addresses: vec![utils::new_network_address()],
        };

        assert!(name == validator.name());
        assert!(GDEXAddress::from(kp.public()) == validator.gdex_address());
        assert!(validator.stake() == stake);
        assert!(validator.delegation() == delegation);

        let _ = ValidatorInfo::voting_rights(&[validator]);
    }
}
