// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use super::Config;
use gdex_types::{account::AuthorityKeyPair, committee::StakeUnit, crypto::GDEXAddress, serialization::KeyPairBase64};
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Serialize, Deserialize)]
pub struct GenesisConfig {
    pub validator_genesis_info: Option<Vec<ValidatorGenesisInfo>>,
    pub committee_size: usize,
    pub accounts: Vec<AccountConfig>,
}

impl Config for GenesisConfig {}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidatorGenesisInfo {
    #[serde_as(as = "KeyPairBase64")]
    pub key_pair: AuthorityKeyPair,
    pub network_address: Multiaddr,
    pub stake: StakeUnit,
    pub narwhal_primary_to_primary: Multiaddr,
    pub narwhal_worker_to_primary: Multiaddr,
    pub narwhal_primary_to_worker: Multiaddr,
    pub narwhal_worker_to_worker: Multiaddr,
    pub narwhal_consensus_address: Multiaddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "GDEXAddress::optional_address_as_hex",
        deserialize_with = "GDEXAddress::optional_address_from_hex"
    )]
    pub address: Option<GDEXAddress>,
}

const DEFAULT_NUMBER_OF_AUTHORITIES: usize = 4;
const DEFAULT_NUMBER_OF_ACCOUNT: usize = 5;

impl GenesisConfig {
    pub fn for_local_testing() -> Self {
        Self::custom_genesis(DEFAULT_NUMBER_OF_AUTHORITIES, DEFAULT_NUMBER_OF_ACCOUNT)
    }

    pub fn custom_genesis(num_authorities: usize, num_accounts: usize) -> Self {
        assert!(num_authorities > 0, "num_authorities should be larger than 0");

        let mut accounts = Vec::new();
        for _ in 0..num_accounts {
            accounts.push(AccountConfig { address: None })
        }

        Self {
            accounts,
            ..Default::default()
        }
    }
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            validator_genesis_info: None,
            committee_size: DEFAULT_NUMBER_OF_AUTHORITIES,
            accounts: vec![],
        }
    }
}
