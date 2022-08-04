//!
//! account objects are attached to specific Controllers and are
//! responsible for keeping important controller-specific data
//!
use std::{collections::HashMap, fmt::Debug};
use types::{AccountBalance, AccountPubKey, AssetId};

// BankAccount is consumed by the BankController
#[derive(Debug)]
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

// OrderAccount is consumed by the SpotController
#[derive(Debug)]
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

// StakeAccount is consumed by the StakeController
#[derive(Debug)]
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

/// Begin the testing suite for account
#[cfg(test)]
pub mod account_tests {
    use super::*;
    use narwhal_crypto::traits::KeyPair;
    use types::account_test_functions::generate_keypair_vec;

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
