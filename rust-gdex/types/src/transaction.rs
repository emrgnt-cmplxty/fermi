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
    crypto::ToFromBytes,
    error::GDEXError,
    order_book::OrderSide,
    serialization::{Base64, Encoding},
};

pub use crate::proto::*;

// gdex

// mysten
use fastcrypto::{traits::Signer, Digest, Hash, Verifier, DIGEST_LEN};
use narwhal_types::{CertificateDigest, CertificateDigestProto};

// external
use blake2::digest::Update;
use prost::bytes::Bytes;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Cursor;
use std::ops::Deref;

// CONSTANTS

pub const PROTO_VERSION: Version = Version {
    major: 0,
    minor: 0,
    patch: 0,
};

pub const DEFAULT_TRANSACTION_FEE: u64 = 1000;

// DIGEST TYPES

#[derive(Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionDigest([u8; DIGEST_LEN]);

impl fmt::Display for TransactionDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Base64::encode(&self.0).get(0..16).unwrap())
    }
}

impl TransactionDigest {
    pub fn new(val: [u8; DIGEST_LEN]) -> TransactionDigest {
        TransactionDigest(val)
    }

    pub fn get_array(&self) -> [u8; DIGEST_LEN] {
        self.0
    }
}

impl From<TransactionDigest> for Digest {
    fn from(digest: TransactionDigest) -> Self {
        Digest::new(digest.0)
    }
}

// SERIALIZATION

pub fn serialize_protobuf<T>(proto_message: &T) -> Vec<u8>
where
    T: Message + std::default::Default,
{
    let mut buf = Vec::new();
    buf.reserve(proto_message.encoded_len());
    proto_message.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_protobuf<T: Message + std::default::Default>(buf: &[u8]) -> Result<T, GDEXError> {
    let message_result = T::decode(&mut Cursor::new(buf));
    match message_result {
        Ok(message) => Ok(message),
        Err(..) => Err(GDEXError::DeserializationError),
    }
}

// TRAITS

// TODO this should be doable through proto autogen stuff,
// but couldn't figure that out so for now we define our own trait
pub trait RequestTypeEnum {
    fn request_type_from_i32(value: i32) -> Result<Self, GDEXError>
    where
        Self: Sized;
}

pub trait Request {
    fn get_controller_id() -> i32;
    fn get_request_type_id() -> i32;
}

// INTERFACE

// CONSENSUS TRANSACTION

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsensusTransaction {
    signed_transaction_bytes: Vec<u8>,
}

impl ConsensusTransaction {
    pub fn new(signed_transaction: &SignedTransaction) -> Self {
        ConsensusTransaction {
            signed_transaction_bytes: serialize_protobuf(signed_transaction),
        }
    }

    pub fn deserialize(byte_vec: Vec<u8>) -> Result<Self, GDEXError> {
        match bincode::deserialize(&byte_vec[..]) {
            Ok(result) => Ok(result),
            Err(..) => Err(GDEXError::DeserializationError),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, GDEXError> {
        match bincode::serialize(&self) {
            Ok(result) => Ok(result),
            Err(..) => Err(GDEXError::SerializationError),
        }
    }

    pub fn get_payload(&self) -> Result<SignedTransaction, GDEXError> {
        deserialize_protobuf(&self.signed_transaction_bytes)
    }
}

// SIGNED TRANSACTION

impl SignedTransaction {
    pub fn new(transaction: Transaction, signature: &[u8; 64]) -> Self {
        SignedTransaction {
            transaction: Some(transaction),
            signature: Bytes::from(signature.to_vec()),
        }
    }

    pub fn get_transaction(&self) -> Result<&Transaction, GDEXError> {
        self.transaction.as_ref().ok_or(GDEXError::DeserializationError)
    }

    pub fn get_sender(&self) -> Result<AccountPubKey, GDEXError> {
        self.get_transaction()?.get_sender()
    }

    pub fn get_signature(&self) -> Result<AccountSignature, GDEXError> {
        AccountSignature::from_bytes(&self.signature).map_err(|_e| GDEXError::DeserializationError)
    }

    pub fn get_recent_block_digest(&self) -> Result<CertificateDigest, GDEXError> {
        self.get_transaction()?.get_recent_block_digest()
    }

    pub fn get_transaction_digest(&self) -> Result<TransactionDigest, GDEXError> {
        Ok(self.get_transaction()?.digest())
    }

    pub fn verify_signature(&self) -> Result<(), GDEXError> {
        let transaction = self.get_transaction()?;
        let transaction_digest = transaction.digest();
        let sender = transaction.get_sender()?;
        let signature = self.get_signature()?;
        sender
            .verify(&transaction_digest.get_array()[..], &signature)
            .map_err(|_e| GDEXError::DeserializationError)
    }
}

// TRANSACTION

impl Transaction {
    pub fn new<T: Request + Message + std::default::Default>(
        sender: &AccountPubKey,
        recent_block_hash: CertificateDigest,
        request: &T,
    ) -> Self {
        Transaction {
            version: Some(PROTO_VERSION),
            sender: Bytes::from(sender.as_ref().to_vec()),
            target_controller: T::get_controller_id(),
            request_type: T::get_request_type_id(),
            recent_block_hash: CertificateDigestProto::from(recent_block_hash).digest,
            fee: DEFAULT_TRANSACTION_FEE,
            request_bytes: Bytes::from(serialize_protobuf(request)),
        }
    }

    pub fn set_fee(&mut self, fee: u64) {
        self.fee = fee;
    }

    pub fn get_sender(&self) -> Result<AccountPubKey, GDEXError> {
        AccountPubKey::from_bytes(&self.sender).map_err(|_e| GDEXError::DeserializationError)
    }

    pub fn get_request_type<T: RequestTypeEnum>(&self) -> Result<T, GDEXError> {
        T::request_type_from_i32(self.request_type)
    }

    pub fn get_recent_block_digest(&self) -> Result<CertificateDigest, GDEXError> {
        match self.recent_block_hash.deref().try_into() {
            Ok(digest) => Ok(CertificateDigest::new(digest)),
            Err(..) => Err(GDEXError::DeserializationError),
        }
    }

    pub fn sign(self, sender_kp: &AccountKeyPair) -> Result<SignedTransaction, GDEXError> {
        let transaction_digest = self.digest();
        let signature_result: Result<AccountSignature, signature::Error> =
            sender_kp.try_sign(&transaction_digest.get_array()[..]);
        match signature_result {
            Ok(result) => Ok(SignedTransaction::new(self, &result.sig.to_bytes())),
            Err(..) => Err(GDEXError::SigningError),
        }
    }
}

impl Hash for Transaction {
    type TypedDigest = TransactionDigest;

    fn digest(&self) -> TransactionDigest {
        TransactionDigest::new(fastcrypto::blake2b_256(|hasher| {
            hasher.update(serialize_protobuf(self))
        }))
    }
}

// ENUM CONVERSIONS
// TODO gotta be a better way to do this

pub fn parse_order_side(side: u64) -> Result<OrderSide, GDEXError> {
    match side {
        1 => Ok(OrderSide::Bid),
        2 => Ok(OrderSide::Ask),
        _ => Err(GDEXError::DeserializationError),
    }
}
