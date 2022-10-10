// gdex
use gdex_types::account::AccountPubKey;
// external
use serde::{Deserialize, Serialize};

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
