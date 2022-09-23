// IMPORTS

// crate
use crate::router::ControllerType;

// gdex
use gdex_types::{
    account::AccountPubKey,
    crypto::ToFromBytes,
    error::GDEXError,
    transaction::{Event, EventTypeEnum, Request, RequestTypeEnum, Transaction},
};

// mysten
use narwhal_types::CertificateDigest;

// external
use prost::bytes::Bytes;

// MODULE IMPORTS

#[path = "./generated/bank_proto.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod bank_proto;

pub use bank_proto::*;

// ENUMS

impl RequestTypeEnum for BankRequestType {
    fn request_type_from_i32(value: i32) -> Result<Self, GDEXError> {
        match value {
            0 => Ok(BankRequestType::CreateAsset),
            1 => Ok(BankRequestType::Payment),
            _ => Err(GDEXError::DeserializationError),
        }
    }
}

impl EventTypeEnum for BankEventType {
    fn event_type_from_i32(value: i32) -> Result<Self, GDEXError> {
        match value {
            0 => Ok(BankEventType::AssetCreated),
            1 => Ok(BankEventType::PaymentSuccess),
            _ => Err(GDEXError::DeserializationError),
        }
    }
}

// REQUESTS

// create asset

impl CreateAssetRequest {
    pub fn new(dummy: u64) -> Self {
        CreateAssetRequest { dummy }
    }
}

impl Request for CreateAssetRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Bank as i32
    }
    fn get_request_type_id() -> i32 {
        BankRequestType::CreateAsset as i32
    }
}

// payment

impl PaymentRequest {
    pub fn new(receiver: &AccountPubKey, asset_id: u64, quantity: u64) -> Self {
        PaymentRequest {
            receiver: Bytes::from(receiver.as_ref().to_vec()),
            asset_id,
            quantity,
        }
    }

    pub fn get_receiver(&self) -> Result<AccountPubKey, GDEXError> {
        AccountPubKey::from_bytes(&self.receiver).map_err(|_e| GDEXError::DeserializationError)
    }
}

impl Request for PaymentRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Bank as i32
    }
    fn get_request_type_id() -> i32 {
        BankRequestType::Payment as i32
    }
}

// EVENTS

// asset created

impl AssetCreatedEvent {
    pub fn new(asset_id: u64) -> Self {
        AssetCreatedEvent { asset_id }
    }
}

impl Event for AssetCreatedEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Bank as i32
    }
    fn get_event_type_id() -> i32 {
        BankEventType::AssetCreated as i32
    }
}

// payment

impl PaymentSuccessEvent {
    pub fn new(sender: &AccountPubKey, receiver: &AccountPubKey, asset_id: u64, quantity: u64) -> Self {
        PaymentSuccessEvent {
            sender: Bytes::from(sender.as_ref().to_vec()),
            receiver: Bytes::from(receiver.as_ref().to_vec()),
            asset_id,
            quantity,
        }
    }
}

impl Event for PaymentSuccessEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Bank as i32
    }
    fn get_event_type_id() -> i32 {
        BankEventType::PaymentSuccess as i32
    }
}

// TRANSACTION BUILDERS

// TODO - https://github.com/gdexorg/gdex/issues/168 - add real params
pub fn create_create_asset_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    dummy: u64,
) -> Transaction {
    Transaction::new(sender, recent_block_hash, &CreateAssetRequest::new(dummy))
}

pub fn create_payment_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    receiver: &AccountPubKey,
    asset_id: u64,
    amount: u64,
) -> Transaction {
    Transaction::new(
        sender,
        recent_block_hash,
        &PaymentRequest::new(receiver, asset_id, amount),
    )
}

/// Begin externally available testing functions
#[cfg(any(test, feature = "testing"))]
pub mod bank_controller_test_functions {
    use super::*;
    use fastcrypto::DIGEST_LEN;
    use gdex_types::{account::AccountKeyPair, crypto::KeypairTraits, transaction::SignedTransaction};

    pub const PRIMARY_ASSET_ID: u64 = 0;

    pub fn generate_signed_test_transaction(
        kp_sender: &AccountKeyPair,
        kp_receiver: &AccountKeyPair,
        amount: u64,
    ) -> SignedTransaction {
        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);

        let transaction = create_payment_transaction(
            kp_sender.public(),
            dummy_batch_digest,
            kp_receiver.public(),
            PRIMARY_ASSET_ID,
            amount,
        );

        transaction.sign(kp_sender).unwrap()
    }
}
