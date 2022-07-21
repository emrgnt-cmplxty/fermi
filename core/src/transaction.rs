//!
//! transactions are the base unit fed into the blockchain
//! to trigger state transitions
//!
use gdex_crypto::{
    hash::{CryptoHash, HashValue},
    Signature,
};
use gdex_crypto_derive::{BCSCryptoHash, CryptoHasher};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::SystemTime};
use types::{
    account::{AccountPubKey, AccountSignature},
    asset::AssetId,
    orderbook::OrderSide,
    spot::DiemCryptoMessage,
};

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct CreateAssetRequest {}

#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum OrderRequest {
    Market {
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        quantity: u64,
        ts: SystemTime,
    },

    Limit {
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        price: u64,
        quantity: u64,
        ts: SystemTime,
    },

    Amend {
        id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
        ts: SystemTime,
    },

    CancelOrder {
        id: u64,
        side: OrderSide,
        //ts: SystemTime,
    },
}

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub struct CreateOrderbookRequest {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
}
impl CreateOrderbookRequest {
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId) -> Self {
        CreateOrderbookRequest {
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
pub struct PaymentRequest {
    // storing from here is not redundant as from may not equal sender
    // e.g. we are preserving the possibility of adding re-key functionality
    from: AccountPubKey,
    to: AccountPubKey,
    asset_id: AssetId,
    amount: u64,
}
impl PaymentRequest {
    pub fn new(from: AccountPubKey, to: AccountPubKey, asset_id: AssetId, amount: u64) -> Self {
        PaymentRequest {
            from,
            to,
            asset_id,
            amount,
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
pub struct StakeRequest {
    from: AccountPubKey,
    amount: u64,
}
impl StakeRequest {
    pub fn new(from: AccountPubKey, amount: u64) -> Self {
        StakeRequest { from, amount }
    }

    pub fn get_from(&self) -> &AccountPubKey {
        &self.from
    }

    pub fn get_amount(&self) -> u64 {
        self.amount
    }
}

#[derive(BCSCryptoHash, Copy, Clone, CryptoHasher, Debug, Deserialize, Serialize)]
pub enum TransactionVariant {
    PaymentTransaction(PaymentRequest),
    CreateOrderbookTransaction(CreateOrderbookRequest),
    CreateAssetTransaction(CreateAssetRequest),
    OrderTransaction(OrderRequest),
    StakeAsset(StakeRequest),
}

#[derive(Clone, Debug)]
pub struct TransactionRequest<TransactionVariant>
where
    TransactionVariant: Debug + Clone + CryptoHash + Copy,
{
    transaction: TransactionVariant,
    sender: AccountPubKey,
    transaction_signature: AccountSignature,
}
impl<TransactionVariant> TransactionRequest<TransactionVariant>
where
    TransactionVariant: Debug + Clone + CryptoHash + Copy,
{
    pub fn new(
        transaction: TransactionVariant,
        sender: AccountPubKey,
        transaction_signature: AccountSignature,
    ) -> Self {
        TransactionRequest {
            transaction,
            sender,
            transaction_signature,
        }
    }

    pub fn get_transaction(&self) -> &TransactionVariant {
        &self.transaction
    }

    pub fn get_sender(&self) -> &AccountPubKey {
        &self.sender
    }

    pub fn get_transaction_signature(&self) -> &AccountSignature {
        &self.transaction_signature
    }

    pub fn verify_transaction(&self) -> Result<(), gdex_crypto::error::Error> {
        let transaction_hash: HashValue = self.transaction.hash();
        self.transaction_signature
            .verify(&DiemCryptoMessage(transaction_hash.to_string()), &self.sender)
    }
}

#[cfg(feature = "batch")]
pub fn verify_transaction_batch(
    transaction_requests: &[TransactionRequest<TransactionVariant>],
) -> Result<(), gdex_crypto::error::Error> {
    let mut messages: Vec<DiemCryptoMessage> = Vec::new();
    let mut keys_and_signatures: Vec<(AccountPubKey, AccountSignature)> = Vec::new();

    for transaction_request in transaction_requests.iter() {
        let transaction_hash: HashValue = transaction_request.transaction.hash();
        messages.push(DiemCryptoMessage(transaction_hash.to_string()));
        keys_and_signatures.push((
            *transaction_request.get_sender(),
            transaction_request.get_transaction_signature().clone(),
        ));
    }
    Signature::batch_verify_distinct(&messages, keys_and_signatures)?;
    Ok(())
}
