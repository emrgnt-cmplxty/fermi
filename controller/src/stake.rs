//! Manages the staking of user funds
//!
//! TODO
//! 0.) ADD SIZE CHECKS ON TRANSACTIONS
//!
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use super::bank::BankController;
use gdex_types::{
    account::{AccountPubKey, StakeAccount},
    asset::PRIMARY_ASSET_ID,
    error::GDEXError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The stake controller is responsible for accessing & modifying user balances
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StakeController {
    stake_accounts: HashMap<AccountPubKey, StakeAccount>,
    bank_controller: Arc<Mutex<BankController>>,
    total_staked: u64,
}
impl StakeController {
    pub fn new(bank_controller: Arc<Mutex<BankController>>) -> Self {
        StakeController {
            bank_controller,
            stake_accounts: HashMap::new(),
            total_staked: 0,
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), GDEXError> {
        if self.stake_accounts.contains_key(account_pub_key) {
            Err(GDEXError::AccountCreation)
        } else {
            self.stake_accounts
                .insert(account_pub_key.clone(), StakeAccount::new(account_pub_key.clone()));
            Ok(())
        }
    }

    pub fn get_staked(&self, account_pub_key: &AccountPubKey) -> Result<&u64, GDEXError> {
        let stake_account = self
            .stake_accounts
            .get(account_pub_key)
            .ok_or(GDEXError::AccountLookup)?;
        Ok(stake_account.get_staked_amount())
    }

    // stake funds to participate in consensus
    pub fn stake(&mut self, account_pub_key: &AccountPubKey, amount: u64) -> Result<(), GDEXError> {
        self.bank_controller
            .lock()
            .unwrap()
            .update_balance(account_pub_key, PRIMARY_ASSET_ID, -(amount as i64))?;
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
    pub fn unstake(&mut self, account_pub_key: &AccountPubKey, amount: u64) -> Result<(), GDEXError> {
        self.total_staked -= amount;
        self.bank_controller
            .lock()
            .unwrap()
            .update_balance(account_pub_key, PRIMARY_ASSET_ID, amount as i64)?;
        let stake_account = self
            .stake_accounts
            .get_mut(account_pub_key)
            .ok_or(GDEXError::AccountLookup)?;
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
        let bank_controller = BankController::new();
        Self::new(Arc::new(Mutex::new(bank_controller)))
    }
}

/// Begin the testing suite for account
#[cfg(test)]
pub mod stake_tests {
    use super::*;
    use crate::bank::CREATED_ASSET_BALANCE;
    use gdex_types::account::account_test_functions::generate_keypair_vec;
    use gdex_types::crypto::KeypairTraits;

    const STAKE_AMOUNT: u64 = 1_000;
    #[test]
    fn stake() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(sender.public()).unwrap();
        bank_controller.create_asset(sender.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));
        let mut stake_controller = StakeController::new(Arc::clone(&bank_controller_ref));

        stake_controller.stake(sender.public(), STAKE_AMOUNT).unwrap();
        assert!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(sender.public(), PRIMARY_ASSET_ID)
                .unwrap()
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
    #[test]
    fn stake_empty() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(sender.public()).unwrap();
        bank_controller.create_asset(sender.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));
        let mut stake_controller = StakeController::new(Arc::clone(&bank_controller_ref));

        stake_controller.stake(sender.public(), STAKE_AMOUNT).unwrap();
        assert!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(sender.public(), PRIMARY_ASSET_ID)
                .unwrap()
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
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));
        let mut stake_controller = StakeController::new(Arc::clone(&bank_controller_ref));

        assert!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(sender.public(), PRIMARY_ASSET_ID)
                .unwrap()
                == 0,
            "unexpected balance"
        );
        // staking without funding should create error
        let second = generate_keypair_vec([0; 32]).pop().unwrap();
        stake_controller.stake(&second.public(), STAKE_AMOUNT).unwrap();
    }
}
