//! 
//! TODO
//! 0.) ADD MISSING FEATURES TO ASSET WORKFLOW, LIKE OWNER TOKEN MINTING, VARIABLE INITIAL MINT AMT., ...
//! 1.) MAKE ROBUST ERROR HANDLING FOR ALL FUNCTIONS
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

// TODO #3 //
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
            n_assets: 0,
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
    // TODO #0 & #1 //
    pub fn create_asset(&mut self, owner_pub_key: &AccountPubKey) -> u64 {
        self.asset_id_to_asset.insert(self.n_assets, Asset{asset_id: self.n_assets, asset_addr: self.n_assets, owner_pubkey: *owner_pub_key});
        self.update_balance(owner_pub_key, self.n_assets, CREATED_ASSET_BALANCE as i64);
        self.n_assets += 1;
        self.n_assets - 1
    }

    // TODO #1 //
    pub fn get_balance(&self, account_pub_key: &AccountPubKey, asset_id: AssetId) -> u64 {
        let bank_account: &BankAccount = self.bank_accounts.get(account_pub_key).unwrap();
        *bank_account.get_balances().get(&asset_id).unwrap_or(&0)
    }

    // TODO #1 //
    pub fn update_balance(&mut self, account_pub_key: &AccountPubKey, asset_id: AssetId, amount: i64) {
        let bank_account: &mut BankAccount = &mut self.bank_accounts.get_mut(account_pub_key).unwrap();
        let prev_amount: i64 = *bank_account.get_balances().get(&asset_id).unwrap_or(&0) as i64;
        assert!((prev_amount + amount) >= 0, "Insufficent balance");
        bank_account.set_balance(asset_id, (prev_amount + amount) as u64);
    }

    // TODO #1 //
    pub fn transfer(&mut self, account_pub_key_from: &AccountPubKey, account_pub_key_to:  &AccountPubKey, asset_id: AssetId, amount: u64) {
        assert!(self.get_balance(account_pub_key_from, asset_id)  >= amount, "Insufficent balance");
        self.update_balance(account_pub_key_from, asset_id, -(amount as i64));
        self.update_balance(account_pub_key_to, asset_id, amount as i64);
    }
}
