//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::asset::AssetId;
use crate::crypto::GDEXPublicKey;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

/// The account key logic is fully configurable to allow for agile changes throughout the
/// codebase, by simply changing the specified type here
/// for now we are leveraging the Sui crypto library and only form a local implementations
/// when it gives a clear reduction in overhead and enhanced consistency
pub type ValidatorPubKey = Ed25519PublicKeyLocal;
pub type ValidatorPrivKey = sui_types::crypto::AuthorityPrivateKey;
pub type ValidatorSignature = sui_types::crypto::AuthoritySignature;
pub type ValidatorKeyPair = sui_types::crypto::AuthorityKeyPair;
pub type ValidatorPubKeyBytes = sui_types::crypto::AuthorityPublicKeyBytes;

pub type AccountPubKey = sui_types::crypto::AccountPublicKey;
pub type AccountPrivKey = sui_types::crypto::AccountPrivateKey;
pub type AccountSignature = sui_types::crypto::AccountSignature;
pub type AccountKeyPair = sui_types::crypto::AccountKeyPair;
pub type AccountBalance = u64;

/// create a local representation of the Ed25519PublicKey in order to implement necessary traits
/// such a change is necessary in order to implement the GDEXPublicKey locally, rather than utilize
/// the exposed SuiPublicKey
pub type Ed25519PublicKeyLocal = sui_types::crypto::AuthorityPublicKey;

impl GDEXPublicKey for Ed25519PublicKeyLocal {
    const FLAG: u8 = 0x00;
}

/// BankAccount is consumed by the BankController
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankAccount {
    account_pub_key: AccountPubKey,
    balances: HashMap<AssetId, AccountBalance>,
}
impl BankAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        BankAccount {
            account_pub_key,
            balances: HashMap::new(),
        }
    }

    pub fn get_account_pub_key(&self) -> &AccountPubKey {
        &self.account_pub_key
    }

    pub fn get_balances(&self) -> &HashMap<AssetId, AccountBalance> {
        &self.balances
    }

    pub fn get_balance(&self, asset_id: AssetId) -> &u64 {
        self.balances.get(&asset_id).unwrap_or(&0)
    }

    pub fn set_balance(&mut self, asset_id: AssetId, amount: u64) {
        self.balances.insert(asset_id, amount);
    }
}

/// OrderAccount is consumed by the SpotController
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderAccount {
    account_pub_key: AccountPubKey,
}
impl OrderAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        OrderAccount { account_pub_key }
    }

    pub fn get_account_pub_key(&self) -> &AccountPubKey {
        &self.account_pub_key
    }
}

/// StakeAccount is consumed by the StakeController
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StakeAccount {
    account_pub_key: AccountPubKey,
    staked_amount: u64,
}
impl StakeAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        StakeAccount {
            account_pub_key,
            staked_amount: 0,
        }
    }

    pub fn get_account_pub_key(&self) -> &AccountPubKey {
        &self.account_pub_key
    }

    pub fn get_staked_amount(&self) -> &u64 {
        &self.staked_amount
    }

    pub fn set_staked_amount(&mut self, new_amount: u64) {
        self.staked_amount = new_amount;
    }
}

/// Begin externally available testing functions
#[cfg(any(test, feature = "testing"))]
pub mod account_test_functions {
    use super::*;
    use crate::crypto::KeypairTraits;
    use rand::{rngs::StdRng, SeedableRng};

    pub fn generate_keypair_vec(seed: [u8; 32]) -> Vec<AccountKeyPair> {
        let mut rng = StdRng::from_seed(seed);
        (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
    }
}

/// Begin the testing suite for account
#[cfg(test)]
pub mod account_tests {
    use super::account_test_functions::generate_keypair_vec;
    use super::*;
    use crate::crypto::KeypairTraits;

    #[test]
    pub fn create_bank_account() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let mut bank_account = BankAccount::new(sender.public().clone());

        assert!(bank_account.get_account_pub_key() == &sender.public().clone());

        let new_amount = 1_000;
        bank_account.set_balance(0, new_amount);

        assert!(*bank_account.get_balances().get(&0).unwrap() == new_amount);
        assert!(*bank_account.get_balance(0) == new_amount);
        assert!(*bank_account.get_balance(1) == 0);
    }

    #[test]
    pub fn create_order_account() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let order_account = OrderAccount::new(sender.public().clone());

        assert!(order_account.get_account_pub_key() == &sender.public().clone());
    }

    #[test]
    pub fn create_stake_account() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let mut stake_account = StakeAccount::new(sender.public().clone());

        assert!(stake_account.get_account_pub_key() == &sender.public().clone());

        let new_amount = 1_000;
        stake_account.set_staked_amount(new_amount);

        assert!(*stake_account.get_staked_amount() == new_amount);
    }
}
