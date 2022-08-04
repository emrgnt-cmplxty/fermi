// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sui_types::base_types::{SuiAddress};
use sui_types::committee::StakeUnit;
use sui_types::crypto::{get_key_pair_from_rng, AccountKeyPair, AuthorityKeyPair};
use sui_types::sui_serde::KeyPairBase64;
use sui_config::Config;

#[derive(Serialize, Deserialize)]
pub struct GenesisConfig {
    pub validator_genesis_info: Option<Vec<ValidatorGenesisInfo>>,
    pub committee_size: usize,
    pub accounts: Vec<AccountConfig>,
}

impl Config for GenesisConfig {}

impl GenesisConfig {
    pub fn generate_accounts<R: ::rand::RngCore + ::rand::CryptoRng>(
        &self,
        mut rng: R,
    ) -> Result<Vec<AccountKeyPair>> {
        let mut keys = Vec::new();
        for account in &self.accounts {
            let (address, keypair) = get_key_pair_from_rng(&mut rng);
            keys.push(keypair);
        }
        Ok(keys)
    }
}

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
        serialize_with = "SuiAddress::optional_address_as_hex",
        deserialize_with = "SuiAddress::optional_address_from_hex"
    )]
    pub address: Option<SuiAddress>,
}

const DEFAULT_NUMBER_OF_AUTHORITIES: usize = 4;
const DEFAULT_NUMBER_OF_ACCOUNT: usize = 5;
const DEFAULT_NUMBER_OF_OBJECT_PER_ACCOUNT: usize = 5;

impl GenesisConfig {
    pub fn for_local_testing() -> Self {
        Self::custom_genesis(
            DEFAULT_NUMBER_OF_AUTHORITIES,
            DEFAULT_NUMBER_OF_ACCOUNT,
            DEFAULT_NUMBER_OF_OBJECT_PER_ACCOUNT,
        )
    }

    pub fn custom_genesis(num_authorities: usize, num_accounts: usize, num_objects_per_account: usize) -> Self {
        assert!(num_authorities > 0, "num_authorities should be larger than 0");

        let mut accounts = Vec::new();
        for _ in 0..num_accounts {
            accounts.push(AccountConfig {
                address: None,
            })
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
