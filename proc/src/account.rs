
extern crate engine;

use std::collections::HashMap;
use std::fmt::Debug;
use super::asset::{AssetId};

use diem_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519PrivateKey, Ed25519Signature},
};

pub type AccountPubKey = Ed25519PublicKey;
pub type AccountPrivKey = Ed25519PrivateKey;
pub type AccountSignature = Ed25519Signature;
pub type AccountBalance = u64;

#[derive(Debug)]
pub struct BankAccount {
    pub account_pub_key: AccountPubKey,
    pub balances: HashMap<AssetId, AccountBalance>,
}

impl BankAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        BankAccount{
            account_pub_key,
            balances: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct OrderAccount {
    pub n_orders: u64,
    pub account_pub_key: AccountPubKey,
    pub base_balance: u64,
    pub base_escrow: u64,
    pub quote_balance: u64,
    pub quote_escrow: u64,
}

impl OrderAccount {
    pub fn new(account_pub_key: AccountPubKey, base_balance: u64, quote_balance: u64) -> Self {
        OrderAccount{
            n_orders: 0, 
            account_pub_key,
            base_balance, 
            base_escrow: 0, 
            quote_balance, 
            quote_escrow: 0, 
        }
    }
}

#[derive(Debug)]
pub enum AccountError {
    Creation(String),
    Lookup(String),
    OrderProc(String)
}