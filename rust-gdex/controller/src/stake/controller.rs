//! Manages the staking of user funds
//!
//! TODO
//! 0.) ADD SIZE CHECKS ON TRANSACTIONS
//!
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use crate::bank::controller::BankController;
use crate::controller::Controller;
use crate::router::ControllerRouter;

// gdex
use gdex_types::{
    account::{AccountPubKey, StakeAccount},
    asset::PRIMARY_ASSET_ID,
    crypto::ToFromBytes,
    error::GDEXError,
    store::ProcessBlockStore,
    transaction::Transaction,
};

// mysten

// external
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// CONSTANTS

pub const STAKE_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"STAKECONTROLLERAAAAAAAAAAAAAAAAA";

// INTERFACE

/// The stake controller is responsible for accessing & modifying user balances
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StakeController {
    controller_account: AccountPubKey,
    stake_accounts: HashMap<AccountPubKey, StakeAccount>,
    bank_controller: Arc<Mutex<BankController>>,
    total_staked: u64,
}

impl Default for StakeController {
    fn default() -> Self {
        Self {
            controller_account: AccountPubKey::from_bytes(STAKE_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            stake_accounts: HashMap::new(),
            total_staked: 0,
            bank_controller: Arc::new(Mutex::new(BankController::default())), // TEMPORARY
        }
    }
}

#[async_trait]
impl Controller for StakeController {
    fn initialize(&mut self, master_controller: &ControllerRouter) {
        self.bank_controller = Arc::clone(&master_controller.bank_controller);
    }

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError> {
        self.bank_controller
            .lock()
            .unwrap()
            .create_account(&self.controller_account)?;
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, _transaction: &Transaction) -> Result<(), GDEXError> {
        Err(GDEXError::InvalidRequestTypeError)
    }

    async fn process_end_of_block(
        _controller: Arc<Mutex<Self>>,
        _process_block_store: &ProcessBlockStore,
        _block_number: u64,
    ) {
    }
}

impl StakeController {
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
        self.bank_controller.lock().unwrap().transfer(
            account_pub_key,
            &self.controller_account,
            PRIMARY_ASSET_ID,
            amount,
        )?;
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
        self.bank_controller.lock().unwrap().transfer(
            &self.controller_account,
            account_pub_key,
            PRIMARY_ASSET_ID,
            amount,
        )?;
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

/// Begin the testing suite for account
#[cfg(test)]
pub mod stake_tests {
    use super::*;
    use crate::bank::controller::CREATED_ASSET_BALANCE;
    use gdex_types::account::account_test_functions::generate_keypair_vec;
    use gdex_types::crypto::KeypairTraits;

    const STAKE_AMOUNT: u64 = 1_000;
    #[test]
    fn stake() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let master_controller = ControllerRouter::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();
        let bank_controller_ref = Arc::clone(&master_controller.bank_controller);

        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(sender.public())
            .unwrap();
        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(sender.public())
            .unwrap();

        master_controller
            .stake_controller
            .lock()
            .unwrap()
            .stake(sender.public(), STAKE_AMOUNT)
            .unwrap();
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
            master_controller
                .stake_controller
                .lock()
                .unwrap()
                .get_accounts()
                .keys()
                .len()
                == 1,
            "unexpected number of accounts"
        );
        assert!(
            *master_controller
                .stake_controller
                .lock()
                .unwrap()
                .get_staked(sender.public())
                .unwrap()
                == STAKE_AMOUNT,
            "unexpected stake amount"
        );
        assert!(
            master_controller.stake_controller.lock().unwrap().get_total_staked() == STAKE_AMOUNT,
            "unexpected total staked amount"
        );
    }
    #[test]
    fn stake_empty() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let master_controller = ControllerRouter::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();
        let bank_controller_ref = Arc::clone(&master_controller.bank_controller);

        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(sender.public())
            .unwrap();
        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(sender.public())
            .unwrap();

        master_controller
            .stake_controller
            .lock()
            .unwrap()
            .stake(sender.public(), STAKE_AMOUNT)
            .unwrap();

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
            master_controller
                .stake_controller
                .lock()
                .unwrap()
                .get_accounts()
                .keys()
                .len()
                == 1,
            "unexpected number of accounts"
        );

        assert!(
            *master_controller
                .stake_controller
                .lock()
                .unwrap()
                .get_staked(sender.public())
                .unwrap()
                == STAKE_AMOUNT,
            "unexpected stake amount"
        );

        assert!(
            master_controller.stake_controller.lock().unwrap().get_total_staked() == STAKE_AMOUNT,
            "unexpected total staked amount"
        );
    }

    // TODO #0 //
    #[test]
    #[should_panic]
    fn failed_stake() {
        let sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let master_controller = ControllerRouter::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();
        let bank_controller_ref = Arc::clone(&master_controller.bank_controller);

        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(sender.public())
            .unwrap();
        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(sender.public())
            .unwrap();

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
        master_controller
            .stake_controller
            .lock()
            .unwrap()
            .stake(second.public(), STAKE_AMOUNT)
            .unwrap();
    }
}
