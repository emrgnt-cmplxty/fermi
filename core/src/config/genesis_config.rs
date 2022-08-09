//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/genesis_config.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use super::Config;
use anyhow::Result;
use gdex_types::{
    account::{AccountKeyPair, ValidatorKeyPair},
    committee::StakeUnit,
    crypto::{get_key_pair_from_rng, GDEXAddress},
    serialization::KeyPairBase64,
};
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tracing::info;

/// Created in the genesis ceremony and specifies the parameters for the initial committee
#[derive(Serialize, Deserialize)]
pub struct GenesisConfig {
    pub validator_genesis_info: Option<Vec<ValidatorGenesisInfo>>,
    pub committee_size: usize,
    pub accounts: Vec<AccountConfig>,
}

impl Config for GenesisConfig {}

/// Specifies the validator info at genesis
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidatorGenesisInfo {
    #[serde_as(as = "KeyPairBase64")]
    pub key_pair: ValidatorKeyPair,
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

    pub fn generate_accounts<R: ::rand::RngCore + ::rand::CryptoRng>(&self, mut rng: R) -> Result<Vec<AccountKeyPair>> {
        let mut addresses = Vec::new();

        info!("Creating accounts and gas objects...");

        let mut keys = Vec::new();
        for account in &self.accounts {
            let address = if let Some(address) = account.address {
                address
            } else {
                let (address, keypair) = get_key_pair_from_rng(&mut rng);
                keys.push(keypair);
                address
            };

            addresses.push(address);
        }

        Ok(keys)
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
