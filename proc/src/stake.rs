//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
//!
//! This controller is responsible for managing user staking
//! it relies on BankController and only accesses the 0th (first) created asset
//!
//!
//! TODO
//! 0.) ADD SIZE CHECKS ON TRANSACTIONS
//!
use super::bank::{BankController, PRIMARY_ASSET_ID};
use gdex_types::{
    account::{AccountPubKey, StakeAccount},
    error::ProcError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The stake controller is responsible for accessing & modifying user balances
#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), ProcError> {
        if self.stake_accounts.contains_key(account_pub_key) {
            Err(ProcError::AccountCreation)
        } else {
            self.stake_accounts
                .insert(account_pub_key.clone(), StakeAccount::new(account_pub_key.clone()));
            Ok(())
        }
    }

    pub fn get_staked(&self, account_pub_key: &AccountPubKey) -> Result<&u64, ProcError> {
        let stake_account = self
            .stake_accounts
            .get(account_pub_key)
            .ok_or(ProcError::AccountLookup)?;
        Ok(stake_account.get_staked_amount())
    }

    // stake funds to participate in consensus
    pub fn stake(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        amount: u64,
    ) -> Result<(), ProcError> {
        bank_controller.update_balance(account_pub_key, PRIMARY_ASSET_ID, -(amount as i64))?;
        self.total_staked += amount;
        let lookup = self.stake_accounts.get_mut(account_pub_key);
        match lookup {
            Some(stake_account) => {
                stake_account.set_staked_amount(stake_account.get_staked_amount() + amount as u64);
                Ok(())
            }
            None => {
                let mut new_stake_account = StakeAccount::new(account_pub_key.clone());
                new_stake_account.set_staked_amount(amount);
                self.stake_accounts.insert(account_pub_key.clone(), new_stake_account);
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
    ) -> Result<(), ProcError> {
        self.total_staked -= amount;
        bank_controller.update_balance(account_pub_key, PRIMARY_ASSET_ID, amount as i64)?;
        let stake_account = self
            .stake_accounts
            .get_mut(account_pub_key)
            .ok_or(ProcError::AccountLookup)?;
        stake_account.set_staked_amount(stake_account.get_staked_amount() - amount);
        Ok(())
    }

    pub fn get_accounts(&self) -> &HashMap<AccountPubKey, StakeAccount> {
        &self.stake_accounts
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

/// Begin the testing suite for account
#[cfg(test)]
pub mod stake_tests {
    use super::*;
    use crate::bank::{CREATED_ASSET_BALANCE, PRIMARY_ASSET_ID};
    use gdex_types::account::account_test_functions::generate_keypair_vec;
    use gdex_types::crypto::KeypairTraits;

    const STAKE_AMOUNT: u64 = 1_000;
    #[test]
    fn stake() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(sender.public()).unwrap();
        bank_controller.create_asset(sender.public()).unwrap();

        let mut stake_controller = StakeController::new();
        stake_controller.create_account(sender.public()).unwrap();

        stake_controller
            .stake(&mut bank_controller, sender.public(), STAKE_AMOUNT)
            .unwrap();
        assert!(
            *bank_controller.get_balance(sender.public(), PRIMARY_ASSET_ID).unwrap()
                == CREATED_ASSET_BALANCE - STAKE_AMOUNT,
            "unexpected balance"
        );
        assert!(
            stake_controller.get_accounts().keys().len() == 1,
            "unexpected number of accounts"
        );
        assert!(
            *stake_controller.get_staked(sender.public()).unwrap() == STAKE_AMOUNT,
            "unexpected stake amount"
        );
        assert!(
            stake_controller.get_total_staked() == STAKE_AMOUNT,
            "unexpected total staked amount"
        );
    }

    // TODO #0 //
    #[test]
    #[should_panic]
    fn failed_stake() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(sender.public()).unwrap();
        bank_controller.create_asset(sender.public()).unwrap();

        let mut stake_controller = StakeController::new();
        assert!(
            *bank_controller.get_balance(sender.public(), PRIMARY_ASSET_ID).unwrap() == 0,
            "unexpected balance"
        );
        // staking without funding should create error
        let second = generate_keypair_vec([0; 32]).pop().unwrap();
        stake_controller
            .stake(&mut bank_controller, &second.public(), STAKE_AMOUNT)
            .unwrap();
    }
}
