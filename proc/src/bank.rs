//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
//! This controller is responsible for managing user balances
//! note, other controllers that rely on balance info will consume this
//!
//! TODO
//! 0.) ADD MISSING FEATURES TO ASSET WORKFLOW, LIKE OWNER TOKEN MINTING, VARIABLE INITIAL MINT AMT., ...
//! 1.) MAKE ROBUST ERROR HANDLING FOR ALL FUNCTIONS ~~ DONE
//! 2.) ADD OWNER FUNCTIONS
//! 3.) BETTER BANK ACCOUNT PUB KEY HANDLING SYSTEM & ADDRESS
use gdex_types::{AccountPubKey, Asset, AssetId, BankAccount, ProcError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO #0 //
pub const PRIMARY_ASSET_ID: u64 = 0;
pub const CREATED_ASSET_BALANCE: u64 = 1_000_000_000_000_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankController {
    asset_id_to_asset: HashMap<AssetId, Asset>,
    bank_accounts: HashMap<AccountPubKey, BankAccount>,
    n_assets: u64,
}

impl BankController {
    pub fn new() -> Self {
        BankController {
            asset_id_to_asset: HashMap::new(),
            bank_accounts: HashMap::new(),
            n_assets: 0,
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), ProcError> {
        // do not allow double-creation of a single account
        if self.bank_accounts.contains_key(account_pub_key) {
            Err(ProcError::AccountCreation)
        } else {
            self.bank_accounts
                .insert(account_pub_key.clone(), BankAccount::new(account_pub_key.clone()));
            Ok(())
        }
    }

    fn check_account_exists(&self, account_pub_key: &AccountPubKey) -> Result<(), ProcError> {
        self.bank_accounts
            .get(account_pub_key)
            .ok_or(ProcError::AccountLookup)?;
        Ok(())
    }

    // TODO #0  //
    pub fn create_asset(&mut self, owner_pub_key: &AccountPubKey) -> Result<(), ProcError> {
        // special handling for genesis
        // an account must be created in this instance
        // since account creation is gated by receipt and balance of primary blockchain asset
        if self.n_assets == 0 {
            self.create_account(owner_pub_key)?
        }
        // throw error if attempting to create asset prior to account creation
        self.check_account_exists(owner_pub_key)?;

        self.asset_id_to_asset.insert(
            self.n_assets,
            Asset {
                asset_id: self.n_assets,
                owner_pubkey: owner_pub_key.clone(),
            },
        );

        self.update_balance(owner_pub_key, self.n_assets, CREATED_ASSET_BALANCE as i64)?;
        // increment asset counter & return less the increment
        self.n_assets += 1;
        Ok(())
    }

    pub fn get_balance(&self, account_pub_key: &AccountPubKey, asset_id: AssetId) -> Result<&u64, ProcError> {
        let bank_account = self
            .bank_accounts
            .get(account_pub_key)
            .ok_or(ProcError::AccountLookup)?;
        Ok(bank_account.get_balances().get(&asset_id).unwrap_or(&0))
    }

    pub fn update_balance(
        &mut self,
        account_pub_key: &AccountPubKey,
        asset_id: AssetId,
        amount: i64,
    ) -> Result<(), ProcError> {
        let bank_account = self
            .bank_accounts
            .get_mut(account_pub_key)
            .ok_or(ProcError::AccountLookup)?;
        let prev_amount: i64 = *bank_account.get_balances().get(&asset_id).unwrap_or(&0) as i64;
        // return error if insufficient user balance
        if (prev_amount + amount) < 0 {
            return Err(ProcError::PaymentRequest);
        };

        bank_account.set_balance(asset_id, (prev_amount + amount) as u64);
        Ok(())
    }

    pub fn transfer(
        &mut self,
        account_pub_key_from: &AccountPubKey,
        account_pub_key_to: &AccountPubKey,
        asset_id: AssetId,
        amount: u64,
    ) -> Result<(), ProcError> {
        // return error if insufficient user balance
        let balance = *self.get_balance(account_pub_key_from, asset_id)?;
        if balance < amount {
            return Err(ProcError::PaymentRequest);
        };

        if self.check_account_exists(account_pub_key_to).is_err() {
            if asset_id == 0 {
                self.create_account(account_pub_key_to)?
            } else {
                return Err(ProcError::AccountLookup);
            }
        }

        self.update_balance(account_pub_key_from, asset_id, -(amount as i64))?;
        self.update_balance(account_pub_key_to, asset_id, amount as i64)?;
        Ok(())
    }
}

impl Default for BankController {
    fn default() -> Self {
        Self::new()
    }
}
