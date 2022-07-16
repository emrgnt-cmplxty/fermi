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
    pub account_pub_key: AccountPubKey,
    pub balances: HashMap<AssetId, AccountBalance>,
}

impl BankAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        BankAccount{
            account_pub_key,
            balances: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct OrderAccount {
    pub account_pub_key: AccountPubKey,
}

impl OrderAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        OrderAccount{
            account_pub_key,
        }
    }
}
