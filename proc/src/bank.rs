//! 
//! TODO
//! 0.) ADD MISSING FEATURES TO ASSET WORKFLOW, LIKE OWNER TOKEN MINTING, VARIABLE INITIAL MINT AMT., ...
//! 1.) MAKE ROBUST ERROR HANDLING FOR ALL FUNCTIONS ~~ DONE
//! 2.) ADD OWNER FUNCTIONS
//! 3.) BETTER BANK ACCOUNT PUB KEY HANDLING SYSTEM & ADDRESS
//! 
extern crate types;

use std::collections::HashMap;

use super::account::{BankAccount};
use types::{
    account::{AccountError, AccountPubKey},
    asset::{Asset, AssetId}
};

// TODO #0 //
pub const CREATED_ASSET_BALANCE: u64 = 1_000_000_000_000;

// The stake asset is used in consensus and is the primary asset of the blockchain 
// This asset will be used for transaction payment 
// Further, it is used in other balance related gating 
pub const STAKE_ASSET_ID: u64 = 0;

// The bank controller is responsible for accessing & modifying user balances 
pub struct BankController
{
    asset_id_to_asset: HashMap<AssetId, Asset>,
    bank_accounts: HashMap<AccountPubKey, BankAccount>,
    n_assets: u64,
}

impl BankController
{
    pub fn new() -> Self {
        BankController{
            asset_id_to_asset: HashMap::new(),
            bank_accounts: HashMap::new(),
            n_assets: 0,
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        if self.bank_accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.bank_accounts.insert(*account_pub_key, BankAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn check_account_exists(&self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        self.bank_accounts.get(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        Ok(())
    }
    // TODO #0  //
    pub fn create_asset(&mut self, owner_pub_key: &AccountPubKey) -> Result<u64, AccountError> {
        // special handling for genesis
        if self.n_assets == 0 {
            self.create_account(owner_pub_key)?
        }
        // throw error if attempting to create asset prior to creating account
        self.check_account_exists(owner_pub_key)?;

        self.asset_id_to_asset.insert(self.n_assets, Asset{asset_id: self.n_assets, asset_addr: self.n_assets, owner_pubkey: *owner_pub_key});
        self.update_balance(owner_pub_key, self.n_assets, CREATED_ASSET_BALANCE as i64)?;
        // increment asset counter & return less the increment
        self.n_assets += 1;
        Ok(self.n_assets - 1)
    }

    pub fn get_balance(&self, account_pub_key: &AccountPubKey, asset_id: AssetId) -> Result<u64, AccountError> {
        let bank_account: &BankAccount = self.bank_accounts.get(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        Ok(*bank_account.get_balances().get(&asset_id).unwrap_or(&0))
    }

    pub fn update_balance(&mut self, account_pub_key: &AccountPubKey, asset_id: AssetId, amount: i64) -> Result<(), AccountError> {
        let bank_account: &mut BankAccount = self.bank_accounts.get_mut(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        let prev_amount: i64 = *bank_account.get_balances().get(&asset_id).unwrap_or(&0) as i64;
        // return error if insufficient user balance
        if (prev_amount + amount) < 0  {
            return Err(AccountError::Payment("Insufficent balance".to_string()));
        };

        bank_account.set_balance(asset_id, (prev_amount + amount) as u64);
        return Ok(())
    }

    pub fn transfer(&mut self, account_pub_key_from: &AccountPubKey, account_pub_key_to:  &AccountPubKey, asset_id: AssetId, amount: u64)  -> Result<(), AccountError> {
        // return error if insufficient user balance
        let balance = self.get_balance(account_pub_key_from, asset_id)?;
        if balance < amount {
            return Err(AccountError::Payment("Insufficent balance".to_string()));
        };

        if self.check_account_exists(&account_pub_key_to).is_err() {
            if asset_id == 0 { self.create_account(account_pub_key_to)? } else { return Err(AccountError::Payment("First create account".to_string())) }
        }

        self.update_balance(account_pub_key_from, asset_id, -(amount as i64))?;
        self.update_balance(account_pub_key_to, asset_id, amount as i64)?;
        Ok(())
    }
}
