//! 
//! transactions are the base unit fed into the blockchain
//! to trigger state transitions
//! 
extern crate engine;

use engine::orders::OrderRequest;
use gdex_crypto::{Signature, hash::{CryptoHash, HashValue}};
use gdex_crypto_derive::{BCSCryptoHash, CryptoHasher};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use types::{
    account::{AccountPubKey, AccountSignature},
    asset::AssetId,
    spot::DiemCryptoMessage,
};

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct Payment 
{
    // storing from here is not redundant as from may not equal sender
    // e.g. we are preserving the possibility of adding re-key functionality
    from: AccountPubKey,
    to: AccountPubKey,
    asset_id: AssetId,
    amount: u64,
}
impl Payment {
    pub fn new(from: AccountPubKey, to: AccountPubKey, asset_id: AssetId, amount: u64) -> Self {
        Payment {
            from,
            to,
            asset_id,
            amount
        }
    }

    pub fn get_from(&self) -> &AccountPubKey {
        &self.from
    }

    pub fn get_to(&self) -> &AccountPubKey {
        &self.to
    }

    pub fn get_asset_id(&self) -> AssetId {
        self.asset_id
    }

    pub fn get_amount(&self) -> u64 {
        self.amount
    }
}
#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct CreateAsset 
{
}

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct Order 
{
    order_request: OrderRequest,
}
impl Order {
    pub fn new(order_request: OrderRequest) -> Self {
        Order {
            order_request,
        }
    }

    pub fn get_order_request(&self) -> &OrderRequest {
        &self.order_request
    }
}

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct Stake 
{
    from: AccountPubKey,
    amount: u64,
}
impl Stake {
    pub fn new(from: AccountPubKey, amount: u64) -> Self {
        Stake {
            from,
            amount
        }
    }

    pub fn get_from(&self) -> &AccountPubKey {
        &self.from
    }

    pub fn get_amount(&self) -> u64 {
        self.amount
    }
}

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct CreateOrderBook 
{
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
}
impl CreateOrderBook {
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId) -> Self {
        CreateOrderBook {
            base_asset_id,
            quote_asset_id,
        }
    }

    pub fn get_base_asset_id(&self) -> AssetId {
        self.base_asset_id
    }

    pub fn get_quote_asset_id(&self) -> AssetId {
        self.quote_asset_id
    }
}

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub enum TxnVariant {
    PaymentTransaction(Payment),
    CreateOrderbookTransaction(CreateOrderBook),
    CreateAssetTransaction(CreateAsset),
    OrderTransaction(Order),
    StakeAssetTransaction(Stake),
}

#[derive(Clone)]
pub struct TxnRequest <TxnVariant>
where
    TxnVariant: Debug + Clone + CryptoHash + Copy,
{
    txn: TxnVariant,
    sender: AccountPubKey,
    txn_signature: AccountSignature,
}
impl <TxnVariant> TxnRequest <TxnVariant>
where
    TxnVariant: Debug + Clone + CryptoHash + Copy,
{
    pub fn new(txn: TxnVariant, sender: AccountPubKey, txn_signature: AccountSignature) -> Self {
        TxnRequest {
            txn,
            sender,
            txn_signature
        }
    }

    pub fn get_txn(&self) -> &TxnVariant {
        &self.txn
    }

    pub fn get_sender(&self) -> &AccountPubKey {
        &self.sender
    }

    pub fn get_txn_signature(&self) -> &AccountSignature {
        &self.txn_signature
    }

    pub fn verify_transaction(&self) -> Result<(), gdex_crypto::error::Error> {
        let txn_hash: HashValue = self.txn.hash();
        self.txn_signature.verify(&DiemCryptoMessage(txn_hash.to_string()), &self.sender)
    }
}

#[cfg(feature = "batch")]
pub fn verify_transaction_batch(txn_requests: &Vec<TxnRequest<TxnVariant>>) -> Result<(), gdex_crypto::error::Error>  {

    let mut messages: Vec<DiemCryptoMessage> = Vec::new();
    let mut keys_and_signatures: Vec<(AccountPubKey, AccountSignature)> = Vec::new();

    for txn_request in txn_requests.iter() {
        let txn_hash: HashValue = txn_request.txn.hash();
        messages.push(DiemCryptoMessage(txn_hash.to_string()));
        keys_and_signatures.push((*txn_request.get_sender(), txn_request.get_txn_signature().clone()));
    }
    Signature::batch_verify_distinct(&messages, keys_and_signatures)?;
    Ok(())
}