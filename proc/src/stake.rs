//!
//! this controller is responsible for managing user staking
//! it relies on BankController and only accesses the 0th (first) created asset
//!
//!
//! TODO
//! 0.) ADD SIZE CHECKS ON TRANSACTIONS
//!
extern crate types;

use super::account::StakeAccount;
use super::bank::{BankController, PRIMARY_ASSET_ID};
use std::collections::HashMap;
use types::{account::AccountPubKey, error::GDEXError};

// The stake controller is responsible for accessing & modifying user balances
pub struct StakeController {
    stake_accounts: HashMap<AccountPubKey, StakeAccount>,
    total_staked: u64,
}
impl StakeController {
    pub fn new() -> Self {
        StakeController {
            stake_accounts: HashMap::new(),
            total_staked: 0,
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), GDEXError> {
        if self.stake_accounts.contains_key(account_pub_key) {
            Err(GDEXError::AccountCreation("Account already exists!".to_string()))
        } else {
            self.stake_accounts
                .insert(*account_pub_key, StakeAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn get_staked(&self, account_pub_key: &AccountPubKey) -> Result<u64, GDEXError> {
        let stake_account = self
            .stake_accounts
            .get(account_pub_key)
            .ok_or_else(|| GDEXError::AccountLookup("Failed to find account".to_string()))?;
        Ok(stake_account.get_staked_amount())
    }

    // stake funds to participate in consensus
    pub fn stake(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        amount: u64,
    ) -> Result<(), GDEXError> {
        bank_controller.update_balance(account_pub_key, PRIMARY_ASSET_ID, -(amount as i64))?;
        self.total_staked += amount;
        let lookup = self.stake_accounts.get_mut(account_pub_key);
        match lookup {
            Some(stake_account) => {
                stake_account.set_staked_amount(stake_account.get_staked_amount() + amount as u64);
                Ok(())
            }
            None => {
                let mut new_stake_account = StakeAccount::new(*account_pub_key);
                new_stake_account.set_staked_amount(amount);
                self.stake_accounts.insert(*account_pub_key, new_stake_account);
                Ok(())
            }
        }
    }

    // TODO #0 //
    pub fn unstake(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        amount: u64,
    ) -> Result<(), GDEXError> {
        self.total_staked -= amount;
        bank_controller.update_balance(account_pub_key, PRIMARY_ASSET_ID, amount as i64)?;
        let stake_account = self
            .stake_accounts
            .get_mut(account_pub_key)
            .ok_or_else(|| GDEXError::AccountLookup("Failed to find account".to_string()))?;
        stake_account.set_staked_amount(stake_account.get_staked_amount() - amount);
        Ok(())
    }

    pub fn get_total_staked(&self) -> u64 {
        self.total_staked
    }
}

impl Default for StakeController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::account::generate_key_pair;
    use super::super::bank::{CREATED_ASSET_BALANCE, PRIMARY_ASSET_ID};
    use super::*;

    const STAKE_AMOUNT: u64 = 1_000;
    #[test]
    fn stake() {
        let (account_pub_key, _private_key) = generate_key_pair();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key).unwrap();

        let mut stake_controller = StakeController::new();
        stake_controller.create_account(&account_pub_key).unwrap();

        stake_controller
            .stake(&mut bank_controller, &account_pub_key, STAKE_AMOUNT)
            .unwrap();
        assert_eq!(
            bank_controller.get_balance(&account_pub_key, PRIMARY_ASSET_ID).unwrap(),
            CREATED_ASSET_BALANCE - STAKE_AMOUNT
        );
        assert_eq!(stake_controller.get_staked(&account_pub_key).unwrap(), STAKE_AMOUNT);
        assert_eq!(stake_controller.get_total_staked(), STAKE_AMOUNT);
    }

    // TODO #0 //
    #[test]
    #[should_panic]
    fn failed_stake() {
        let (account_pub_key, _private_key) = generate_key_pair();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key).unwrap();

        let mut stake_controller = StakeController::new();
        assert_eq!(
            bank_controller.get_balance(&account_pub_key, PRIMARY_ASSET_ID).unwrap(),
            0
        );
        // staking without funding should create error
        let (second_account_pub_key, _private_key) = generate_key_pair();
        stake_controller
            .stake(&mut bank_controller, &second_account_pub_key, STAKE_AMOUNT)
            .unwrap();
    }
}
