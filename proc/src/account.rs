use std::{
    collections::HashMap,
    fmt::Debug
};

use types::{
    account::{AccountPubKey, AccountBalance},
    asset::{AssetId}
};

#[derive(Debug)]
pub struct BankAccount {
    account_pub_key: AccountPubKey,
    balances: HashMap<AssetId, AccountBalance>,
}

impl BankAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        BankAccount{
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

    pub fn set_balance(&mut self, asset_id: AssetId, amount: u64) {
        self.balances.insert(asset_id, amount);
    }
}

#[derive(Debug)]
pub struct OrderAccount {
    account_pub_key: AccountPubKey,
}

impl OrderAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        OrderAccount{
            account_pub_key,
        }
    }

    pub fn get_account_pub_key(&self) -> &AccountPubKey {
        &self.account_pub_key
    }
}

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

    pub fn get_staked_amount(&self) -> u64 {
        self.staked_amount
    }

    pub fn set_staked_amount(&mut self, new_amount: u64) {
        self.staked_amount = new_amount;
    }
}


