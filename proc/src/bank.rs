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
pub const CREATED_ASSET_BALANCE: u64 = 1_000_000_000_000;

pub const STAKE_ASSET_ID: u64 = 0;

pub const BANK_ACCOUNT_BYTES: [u8; 32] = [
    215,  90, 152,   1, 130, 177,  10, 183, 213,  75, 254, 211, 201, 100,   7,  58,
    14, 225, 114, 243, 218, 166,  35,  37, 175,   2,  26, 104, 247,   7,   81, 26
];

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
        let mut bank_controller: BankController = BankController{
            asset_id_to_asset: HashMap::new(),
            bank_accounts: HashMap::new(),
            n_assets: 0
        };

        let bank_pub_key: AccountPubKey = AccountPubKey::from_bytes_unchecked(&BANK_ACCOUNT_BYTES).unwrap();

        bank_controller.create_account(&bank_pub_key).unwrap();
        bank_controller.create_asset(&bank_pub_key);
        bank_controller
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        if self.bank_accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.bank_accounts.insert(*account_pub_key, BankAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn create_asset(&mut self, owner_pub_key: &AccountPubKey) -> u64 {
        self.asset_id_to_asset.insert(self.n_assets, Asset{asset_id: self.n_assets, asset_addr: self.n_assets, owner_pubkey: *owner_pub_key});
        self.update_balance(owner_pub_key, self.n_assets, CREATED_ASSET_BALANCE as i64);
        self.n_assets += 1;
        self.n_assets - 1
    }

    pub fn get_balance(&self, account_pub_key: &AccountPubKey, asset_id: AssetId) -> u64 {
        let bank_account: &BankAccount = self.bank_accounts.get(account_pub_key).unwrap();
        *bank_account.balances.get(&asset_id).unwrap_or(&0)
    }

    pub fn update_balance(&mut self, account_pub_key: &AccountPubKey, asset_id: AssetId, amount: i64) {
        let bank_account: &mut BankAccount = &mut self.bank_accounts.get_mut(account_pub_key).unwrap();
        let prev_amount: i64 = *bank_account.balances.get(&asset_id).unwrap_or(&0) as i64;
        bank_account.balances.insert(asset_id, (prev_amount + amount) as u64);
    }

    pub fn transfer(&mut self, account_pub_key_from: &AccountPubKey, account_pub_key_to:  &AccountPubKey, asset_id: AssetId, amount: u64) {
        assert!(self.get_balance(account_pub_key_from, asset_id)  > amount);
        self.update_balance(account_pub_key_from, asset_id, -(amount as i64));
        self.update_balance(account_pub_key_to, asset_id, amount as i64);
    }
}
