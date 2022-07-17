//! 
//! TODO
//! 1.) Add error handling
//! 
extern crate types;

use std::collections::HashMap;

use super::account::{StakeAccount};
use super::bank::{
    BankController,
    STAKE_ASSET_ID
};
use types::{
    account::{AccountError, AccountPubKey},
};

// The stake controller is responsible for accessing & modifying user balances 
pub struct StakeController
{
    stake_accounts: HashMap<AccountPubKey, StakeAccount>,
}

impl StakeController
{
    pub fn new() -> Self {
        StakeController {
            stake_accounts: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        if self.stake_accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.stake_accounts.insert(*account_pub_key, StakeAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn get_staked(&self, account_pub_key: &AccountPubKey) -> u64 {
        let stake_account: &StakeAccount = self.stake_accounts.get(account_pub_key).unwrap();
        stake_account.staked_amount
    }

    pub fn stake(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, amount: i64) {
        bank_controller.update_balance(account_pub_key, STAKE_ASSET_ID, -amount);
        let stake_account: &mut StakeAccount = self.stake_accounts.get_mut(account_pub_key).unwrap();
        stake_account.staked_amount += amount as u64;
    }

    pub fn unstake(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, amount: i64) {
        bank_controller.update_balance(account_pub_key, STAKE_ASSET_ID, amount);
        let stake_account: &mut StakeAccount = self.stake_accounts.get_mut(account_pub_key).unwrap();
        stake_account.staked_amount -= amount as u64;
    }
}
