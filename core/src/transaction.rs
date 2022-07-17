extern crate engine;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug};

use diem_crypto::{
    Signature,
    hash::{CryptoHash, HashValue},
};
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use engine::orders::{OrderRequest};
use types::{
    account::{AccountPubKey, AccountSignature},
    asset::{AssetId},
    spot::{TestDiemCrypto}
};

#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Payment 
{
    pub from: AccountPubKey,
    pub to: AccountPubKey,
    pub asset_id: AssetId,
    pub amount: u64
}
#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CreateAsset 
{
}

#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Order 
{
    pub request: OrderRequest
}
#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TxnVariant {
    PaymentTransaction(Payment),
    CreateAssetTransaction(CreateAsset),
    OrderTransaction(Order),
}
pub struct TxnRequest <TxnVariant>
where
    TxnVariant: Debug + Clone + CryptoHash + Copy,
{
    pub txn: TxnVariant,
    pub sender_address: AccountPubKey,
    pub txn_signature: AccountSignature,
}

pub fn verify_transaction<TxnVariant: Debug + Clone + CryptoHash + Copy>(signed_txn: &TxnRequest<TxnVariant>, account_pub_key: &AccountPubKey) {
    let txn_hash: HashValue = signed_txn.txn.hash();
    signed_txn.txn_signature.verify(&TestDiemCrypto(txn_hash.to_string()), &account_pub_key).unwrap()
}