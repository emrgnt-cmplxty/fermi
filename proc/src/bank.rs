//! 
//! TODO
//! 1.) Move asset addr to proper addr
//! 2.) Improve general asset functionality
//! 3.) Make errors more robust, like in fetching non-existent account
//! 
extern crate types;

use std::collections::HashMap;

use super::account::{BankAccount};
use types::{
    account::{AccountError, AccountPubKey},
    asset::{Asset, AssetId}
};

// TODO - change to process this via input
const CREATED_ASSET_BALANCE: i64 = 1_000_000_000_000;

// The bank controller is responsible for accessing & modifying user balances 
pub struct BankController
{
    asset_id_to_asset: HashMap<AssetId, Asset>,
    accounts: HashMap<AccountPubKey, BankAccount>,
    n_assets: u64,
}


impl BankController
{
    pub fn new() -> Self {
        BankController{
            asset_id_to_asset: HashMap::new(),
            accounts: HashMap::new(),
            n_assets: 0
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        if self.accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.accounts.insert(*account_pub_key, BankAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn create_asset(&mut self, owner_pub_key: &AccountPubKey) {
        self.asset_id_to_asset.insert(self.n_assets, Asset{asset_id: self.n_assets, asset_addr: self.n_assets});
        self.update_balance(owner_pub_key, self.n_assets, CREATED_ASSET_BALANCE);
        self.n_assets += 1;
    }

    pub fn get_balance(&self, account_pub_key: &AccountPubKey, asset_id: AssetId) -> u64 {
        let account: &BankAccount = self.accounts.get(account_pub_key).unwrap();
        *account.balances.get(&asset_id).unwrap_or(&0)
    }

    pub fn update_balance(&mut self, account_pub_key: &AccountPubKey, asset_id: AssetId, amount: i64) {
        let account: &mut BankAccount = &mut self.accounts.get_mut(account_pub_key).unwrap();
        let new_amount: i64 = *account.balances.get(&asset_id).unwrap_or(&0) as i64;
        account.balances.insert(asset_id, (new_amount + amount) as u64);
    }

    pub fn transfer(&mut self, account_pub_key_from: &AccountPubKey, account_pub_key_to:  &AccountPubKey, asset_id: AssetId, amount: u64) {
        assert!(self.get_balance(account_pub_key_from, asset_id)  > amount);
        self.update_balance(account_pub_key_from, asset_id, -(amount as i64));
        self.update_balance(account_pub_key_to, asset_id, amount as i64);
    }

}
