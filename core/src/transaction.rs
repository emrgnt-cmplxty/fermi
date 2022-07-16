extern crate engine;

use serde::{Deserialize, Serialize};

use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use engine::orders::{OrderRequest};
use types::account::{AccountPubKey, AccountSignature};
use types::asset::{AssetId};

pub struct Order
{
    pub txn: OrderRequest,
    pub sender_address: AccountPubKey,
    pub txn_signature: AccountSignature,
}
#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub from: AccountPubKey,
    pub to: AccountPubKey,
    pub asset_id: AssetId,
    pub amount: u64
}
pub struct Payment
{
    pub txn: PaymentRequest,
    pub sender_address: AccountPubKey,
    pub txn_signature: AccountSignature,
}

#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub from: AccountPubKey,
}
pub struct CreateAsset
{
    pub txn: CreateAssetRequest,
    pub sender_address: AccountPubKey,
    pub txn_signature: AccountSignature,
}
pub enum Transaction
{
    OrderTransaction(Order),
    PaymentTransaction(Payment),
    CreateAssetTransaction(CreateAsset)
}