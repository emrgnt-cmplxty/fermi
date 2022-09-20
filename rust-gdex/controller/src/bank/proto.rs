// IMPORTS

// gdex
use gdex_types::{
    account::AccountPubKey,
    crypto::ToFromBytes,
    error::GDEXError,
    transaction::{create_transaction, serialize_protobuf, ControllerType, RequestType, Transaction},
};

// mysten
use narwhal_types::CertificateDigest;

// external
use prost::bytes::Bytes;

// MODULE IMPORTS

#[path = "./generated/bank_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod bank_requests;

pub use bank_requests::*;

// PROTO IMPL

impl PaymentRequest {
    pub fn get_receiver(&self) -> Result<AccountPubKey, GDEXError> {
        AccountPubKey::from_bytes(&self.receiver).map_err(|_e| GDEXError::DeserializationError)
    }
}

// REQUEST BUILDERS

pub fn create_create_asset_request(dummy: u64) -> CreateAssetRequest {
    CreateAssetRequest { dummy }
}

pub fn create_payment_request(receiver: &AccountPubKey, asset_id: u64, quantity: u64) -> PaymentRequest {
    PaymentRequest {
        receiver: Bytes::from(receiver.as_ref().to_vec()),
        asset_id,
        quantity,
    }
}

// TRANSACTION BUILDERS

// TODO get rid of dummy thing (pretty gross)
pub fn create_create_asset_transaction(
    sender: AccountPubKey,
    dummy: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    let request = create_create_asset_request(dummy);

    create_transaction(
        sender,
        ControllerType::Bank,
        RequestType::CreateAsset,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
    )
}

pub fn create_payment_transaction(
    sender: AccountPubKey, // TODO can be ref?
    receiver: &AccountPubKey,
    asset_id: u64,
    amount: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    let request = create_payment_request(receiver, asset_id, amount);

    create_transaction(
        sender,
        ControllerType::Bank,
        RequestType::Payment,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
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
        // TODO replace this with latest
        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);

        let fee: u64 = 1000;
        let transaction = create_payment_transaction(
            kp_sender.public().clone(),
            kp_receiver.public(),
            PRIMARY_ASSET_ID,
            amount,
            fee,
            dummy_batch_digest,
        );

        transaction.sign(kp_sender).unwrap()
    }
}
