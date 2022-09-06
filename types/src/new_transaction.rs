//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
//! The transaction class is responsible for parsing client interactions
//! each valid transaction corresponds to a unique state transition within
//! the space of allowable blockchain transitions

// IMPORTS

// crate
use crate::{
    account::{AccountKeyPair, AccountPubKey, AccountSignature},
    error::GDEXError,
    serialization::{Base64, Encoding},
    crypto::{ToFromBytes},
};

pub use crate::proto::{
    Controller, RequestType, Version, NewSignedTransaction,
    NewTransaction, PaymentRequest, CreateAssetRequest,
    CreateOrderbookRequest, MarketOrderRequest, LimitOrderRequest,
    UpdateOrderRequest, CancelOrderRequest
};

// gdex

// mysten
use narwhal_crypto::{
    Verifier,
    DIGEST_LEN,
    traits::Signer
};
use narwhal_types::{CertificateDigest, CertificateDigestProto};

// external
use prost::bytes::Bytes;
use std::io::Cursor;
use std::ops::Deref;
use prost::Message;
use blake2::digest::Update;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
};

// CONSTANTS

pub const PROTO_VERSION: Version = Version { major: 0, minor: 0, patch: 0 };


// DIGEST TYPES

#[derive(Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NewTransactionDigest([u8; DIGEST_LEN]);

impl fmt::Display for NewTransactionDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Base64::encode(&self.0).get(0..16).unwrap())
    }
}

impl NewTransactionDigest {
    pub fn new(val: [u8; DIGEST_LEN]) -> NewTransactionDigest {
        NewTransactionDigest(val)
    }

    pub fn get_array(&self) -> [u8; DIGEST_LEN] {
        self.0
    }
}

// INTERFACE

pub fn serialize_protobuf<T>(
    proto_message: &T
) -> Vec<u8> where T: Message + std::default::Default {
    let mut buf = Vec::new();
    buf.reserve(proto_message.encoded_len());
    proto_message.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_protobuf<T: Message + std::default::Default>(
    buf: &[u8]
) -> Result<T, GDEXError> {
    let message_result = T::decode(&mut Cursor::new(buf));
    match message_result {
        Ok(message) => Ok(message),
        Err(..) => Err(GDEXError::DeserializationError)
    }
}

// SIGNED TRANSACTION

pub fn create_signed_transaction(
    transaction: NewTransaction,
    signature: &[u8; 64]
) -> NewSignedTransaction {
    NewSignedTransaction {
        transaction: Some(transaction),
        signature: Bytes::from(signature.to_vec())
    }
}

// TRANSACTION

pub fn create_transaction(
    sender: AccountPubKey,
    target_controller: Controller,
    request_type: RequestType,
    recent_block_hash: CertificateDigest,
    gas: u64,
    request_bytes: Vec<u8>
) -> NewTransaction {
    NewTransaction {
        version: Some(PROTO_VERSION),
        sender: Bytes::from(sender.as_ref().to_vec()),
        target_controller: target_controller as i32,
        request_type: request_type as i32,
        recent_block_hash: CertificateDigestProto::from(recent_block_hash).digest,
        gas,
        request_bytes: Bytes::from(request_bytes)
    }
}

// BANK REQUESTS

pub fn create_create_asset_request(
    dummy: u64,
) -> CreateAssetRequest {
    CreateAssetRequest {
        dummy
    }
}

pub fn create_payment_request(
    receiver: &AccountPubKey,
    asset_id: u64,
    amount: u64,
) -> PaymentRequest {
    PaymentRequest {
        receiver: Bytes::from(receiver.as_ref().to_vec()),
        asset_id,
        amount
    }
}

// SPOT REQUESTS

pub fn create_create_orderbook_request(
    base_asset_id: u64,
    quote_asset_id: u64,
) -> CreateOrderbookRequest {
    CreateOrderbookRequest {
        base_asset_id,
        quote_asset_id
    }
}

pub fn create_market_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    quantity: u64,
    local_timestamp: u64,
) -> MarketOrderRequest {
    MarketOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        quantity,
        local_timestamp
    }
}

pub fn create_limit_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    local_timestamp: u64,
) -> LimitOrderRequest {
    LimitOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        local_timestamp
    }
}

pub fn create_update_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    local_timestamp: u64,
    order_id: u64
) -> UpdateOrderRequest {
    UpdateOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        local_timestamp,
        order_id
    }
}

pub fn create_cancel_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    local_timestamp: u64,
    order_id: u64
) -> CancelOrderRequest {
    CancelOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        local_timestamp,
        order_id
    }
}

// TRANSACTION BUILDERS

pub fn new_create_payment_transaction(
    sender: AccountPubKey, // TODO can be ref?
    receiver: &AccountPubKey,
    asset_id: u64,
    amount: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_payment_request(
        receiver,
        asset_id,
        amount
    );

    create_transaction(
        sender,
        Controller::Bank,
        RequestType::Payment,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    )
}

// TODO get rid of dummy thing (pretty gross)
pub fn new_create_create_asset_transaction(
    sender: AccountPubKey,
    dummy: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_create_asset_request(dummy);
    
    create_transaction(
        sender,
        Controller::Bank,
        RequestType::CreateAsset,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    )
}

pub fn new_create_create_orderbook_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_create_orderbook_request(
        base_asset_id,
        quote_asset_id
    );

    create_transaction(
        sender,
        Controller::Spot,
        RequestType::CreateOrderbook,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    ) 
}


pub fn new_create_market_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    quantity: u64,
    local_timestamp: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_market_order_request(
        base_asset_id,
        quote_asset_id,
        side,
        quantity,
        local_timestamp
    );
    
    create_transaction(
        sender,
        Controller::Spot,
        RequestType::MarketOrder,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    )
}

pub fn new_create_limit_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    local_timestamp: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_limit_order_request(
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        local_timestamp,
    );
    
    create_transaction(
        sender,
        Controller::Spot,
        RequestType::LimitOrder,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    )
}

pub fn new_create_update_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    local_timestamp: u64,
    order_id: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_update_order_request(
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        local_timestamp,
        order_id
    );
    
    create_transaction(
        sender,
        Controller::Spot,
        RequestType::UpdateOrder,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    )
}

pub fn new_create_cancel_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    local_timestamp: u64,
    order_id: u64,
    gas: u64,
    recent_block_hash: CertificateDigest,
) -> NewTransaction {
    let request = create_cancel_order_request(
        base_asset_id,
        quote_asset_id,
        side,
        local_timestamp,
        order_id,
    );
    
    create_transaction(
        sender,
        Controller::Spot,
        RequestType::CancelOrder,
        recent_block_hash,
        gas,
        serialize_protobuf(&request)
    )
}

// GETTERS

pub fn get_signed_transaction_body(
    signed_transaction: &NewSignedTransaction
) -> Result<&NewTransaction, GDEXError> {
    match signed_transaction.transaction {
        None => Err(GDEXError::DeserializationError),
        Some(_) => Ok(signed_transaction.transaction.as_ref().unwrap())
    }
}

pub fn get_signed_transaction_sender(
    signed_transaction: &NewSignedTransaction
) -> Result<AccountPubKey, GDEXError> {
    let transaction = get_signed_transaction_body(signed_transaction)?;
    let sender_result = AccountPubKey::from_bytes(&transaction.sender);
    match sender_result {
        Ok(result) => Ok(result),
        Err(..) => Err(GDEXError::DeserializationError)
    }
}

pub fn get_signed_transaction_signature(
    signed_transaction: &NewSignedTransaction
) -> Result<AccountSignature, GDEXError> {
    let account_signature_result = AccountSignature::from_bytes(&signed_transaction.signature);
    match account_signature_result {
        Ok(result) => Ok(result),
        Err(..) => Err(GDEXError::DeserializationError)
    }
}

pub fn get_signed_transaction_recent_block_hash(
    signed_transaction: &NewSignedTransaction
) -> Result<CertificateDigest, GDEXError> {
    let transaction = get_signed_transaction_body(signed_transaction)?;
    match transaction.recent_block_hash.deref().try_into() {
        Ok(digest) => Ok(CertificateDigest::new(digest)),
        Err(..) => Err(GDEXError::DeserializationError)
    }
}

pub fn get_signed_transaction_transaction_hash(
    signed_transaction: &NewSignedTransaction
) -> Result<NewTransactionDigest, GDEXError> {
    let transaction = get_signed_transaction_body(signed_transaction)?;
    Ok(hash_transaction(transaction))
}

// HELPERS

pub fn hash_transaction(
    transaction: &NewTransaction
) -> NewTransactionDigest {
    NewTransactionDigest::new(narwhal_crypto::blake2b_256(|hasher| {hasher.update(serialize_protobuf(transaction)) }))
}

pub fn sign_transaction(
    sender_kp: &AccountKeyPair,
    transaction: NewTransaction
) -> Result<NewSignedTransaction, GDEXError> {
    let transaction_hash = hash_transaction(&transaction);
    let signature_result: Result<AccountSignature, signature::Error> = sender_kp.try_sign(&transaction_hash.get_array()[..]);
    match signature_result {
        Ok(result) => Ok(create_signed_transaction(transaction, &result.sig.to_bytes())),
        Err(..) => Err(GDEXError::SigningError)
    }
}

pub fn verify_signature(
    signed_transaction: &NewSignedTransaction
) -> Result<(), GDEXError> {
    let transaction = get_signed_transaction_body(signed_transaction)?;
    let transaction_hash = hash_transaction(&transaction);
    let sender = get_signed_transaction_sender(signed_transaction)?;
    let signature = get_signed_transaction_signature(&signed_transaction)?;
    let verify_signature_result = sender.verify(&transaction_hash.get_array()[..], &signature);
    match verify_signature_result {
        Ok(_) => Ok(()),
        Err(..) => Err(GDEXError::TransactionSignatureVerificationError)
    }
}