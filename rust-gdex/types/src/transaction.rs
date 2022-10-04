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
    utils,
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
use schemars::{gen::SchemaGenerator, schema::Schema, schema_for, JsonSchema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

impl Serialize for SignedTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = serialize_protobuf(self);
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for SignedTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let position: Result<SignedTransaction, GDEXError> = super::transaction::deserialize_protobuf(&bytes);
        match position {
            Ok(p) => Ok(p),
            Err(e) => Err(serde::de::Error::custom(e.to_string())),
        }
    }
}

impl JsonSchema for SignedTransaction {
    fn schema_name() -> String {
        "SignedTransaction".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let root_schema = schema_for!(SignedTransaction);
        root_schema.schema.into()
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

// EXECUTION RESULT

pub type ExecutionEvents = Vec<ExecutionEvent>;
pub type ExecutionResult = Result<(), GDEXError>;

#[derive(Serialize)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(Serialize)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct ExecutedTransaction {
    pub signed_transaction: SignedTransaction,
    pub events: ExecutionEvents,
    pub result: ExecutionResult,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct QueriedTransaction {
    pub executed_transaction: ExecutedTransaction,
    pub transaction_id: String,
}

impl From<ExecutedTransaction> for QueriedTransaction {
    fn from(executed_transaction: ExecutedTransaction) -> Self {
        if let Ok(transaction) = executed_transaction.signed_transaction.get_transaction() {
            let transaction_digest = transaction.digest().get_array();
            QueriedTransaction {
                executed_transaction,
                transaction_id: utils::encode_bytes_hex(transaction_digest),
            }
        } else {
            QueriedTransaction {
                executed_transaction,
                transaction_id: "".to_string(),
            }
        }
    }
}
// EVENT TYPE

// TODO kinda dumb to have 2 different traits for something functionally identical to RequestTypeEnum
pub trait EventTypeEnum {
    fn event_type_from_i32(value: i32) -> Result<Self, GDEXError>
    where
        Self: Sized;
}

// EXECUTION EVENT

pub trait Event {
    fn get_controller_id() -> i32;
    fn get_event_type_id() -> i32;
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, JsonSchema)]
pub struct ExecutionEvent {
    pub controller_id: i32,
    pub event_type: i32,
    pub event_bytes: Vec<u8>,
}

impl ExecutionEvent {
    pub fn new<T: Event + Message + std::default::Default>(event: &T) -> Self {
        ExecutionEvent {
            controller_id: T::get_controller_id(),
            event_type: T::get_event_type_id(),
            event_bytes: serialize_protobuf(event),
        }
    }

    pub fn get_event_type<T: EventTypeEnum>(&self) -> Result<T, GDEXError> {
        T::event_type_from_i32(self.event_type)
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
