//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
//! The transaction class is responsible for parsing client interactions
//! each valid transaction corresponds to a unique state transition within
//! the space of allowable blockchain transitions
use crate::{
    account::{AccountPubKey, AccountSignature},
    asset::{AssetAmount, AssetId, AssetPrice},
    error::GDEXError,
    order_book::OrderId,
    order_book::OrderSide,
    serialization::Base64,
    serialization::Encoding,
};
use blake2::{digest::Update, VarBlake2b};
use narwhal_crypto::{Digest, Hash, Verifier, DIGEST_LEN};
use narwhal_types::CertificateDigest;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

pub const SERIALIZED_TRANSACTION_LENGTH: usize = 280;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateAssetRequest {}

/// A valid payment transaction causes a state transition inside of
/// the BankController object, e.g. it creates a fund transfer from
/// User A to User B provided User A has sufficient funds
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentRequest {
    receiver: AccountPubKey,
    asset_id: AssetId,
    amount: u64,
}

impl PaymentRequest {
    pub fn new(receiver: AccountPubKey, asset_id: AssetId, amount: AssetAmount) -> Self {
        PaymentRequest {
            receiver,
            asset_id,
            amount,
        }
    }

    pub fn get_receiver(&self) -> &AccountPubKey {
        &self.receiver
    }

    pub fn get_asset_id(&self) -> AssetId {
        self.asset_id
    }

    pub fn get_amount(&self) -> AssetAmount {
        self.amount
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateOrderbookRequest {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
}

impl CreateOrderbookRequest {
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId) -> Self {
        Self {
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

/***#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaceLimitOrderRequest {
    side: OrderSide,
    quantity: AssetAmount,
    price: AssetPrice
}

impl PlaceLimitOrderRequest {
    pub fn new (side: OrderSide, quantity: AssetAmount, price: AssetPrice) -> Self {
        Self {
            side,
            quantity,
            price
        }
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn get_quantity(&self) -> AssetAmount {
        self.quantity
    }

    pub fn get_price(&self) -> AssetPrice {
        self.price
    }
}***/

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OrderRequest {
    Market {
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        side: OrderSide,
        quantity: AssetAmount,
        local_timestamp: SystemTime,
    },

    Limit {
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        side: OrderSide,
        price: AssetPrice,
        quantity: AssetAmount,
        local_timestamp: SystemTime,
    },

    Update {
        id: OrderId,
        side: OrderSide,
        price: AssetPrice,
        quantity: AssetAmount,
        local_timestamp: SystemTime,
    },

    CancelOrder {
        id: OrderId,
        side: OrderSide,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionVariant {
    PaymentTransaction(PaymentRequest),
    CreateAssetTransaction(CreateAssetRequest),
    CreateOrderbookTransaction(CreateOrderbookRequest),
    // this handles limit, market, cancel, update
    PlaceOrderTransaction(OrderRequest),
}

/// A transaction for creating a new asset in the BankController
/// GDEX prefix is added because Narwal github contains a Transaction type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    // storing from here is not redundant as from may not equal sender
    // e.g. we are preserving the possibility of adding re-key functionality
    sender: AccountPubKey,
    // it is necessary to pass a recent block hash to make sure that a transaction cannot
    // be duplicated, moreover it is used to gaurantee that a submitted transaction was
    // created within a well designated lookback, TODO - implement such checks in pipeline
    recent_certificate_digest: CertificateDigest,
    variant: TransactionVariant,
}

impl Transaction {
    pub fn new(
        sender: AccountPubKey,
        recent_certificate_digest: CertificateDigest,
        variant: TransactionVariant,
    ) -> Self {
        Transaction {
            sender,
            recent_certificate_digest,
            variant,
        }
    }

    pub fn get_sender(&self) -> &AccountPubKey {
        &self.sender
    }

    pub fn get_recent_certificate_digest(&self) -> &CertificateDigest {
        &self.recent_certificate_digest
    }

    pub fn get_variant(&self) -> &TransactionVariant {
        &self.variant
    }
}

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

fn convert_system_time_to_int(timestamp: SystemTime) -> u128 {
    timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis()
}

impl Hash for Transaction {
    type TypedDigest = TransactionDigest;

    fn digest(&self) -> TransactionDigest {
        match &self.variant {
            TransactionVariant::PaymentTransaction(payment) => {
                let hasher_update = |hasher: &mut VarBlake2b| {
                    hasher.update(self.get_sender().0.to_bytes());
                    hasher.update(payment.get_receiver().0.as_bytes());
                    hasher.update(payment.get_asset_id().to_le_bytes());
                    hasher.update(payment.get_amount().to_le_bytes());
                    hasher.update(self.get_recent_certificate_digest().to_string());
                    // TODO this doesn't really make sense but the contents are private
                };
                TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
            }
            TransactionVariant::CreateAssetTransaction(_create_asset) => {
                let hasher_update = |hasher: &mut VarBlake2b| {
                    hasher.update(self.get_sender().0.to_bytes());
                    hasher.update(self.get_recent_certificate_digest().to_string())
                    // TODO this doesn't really make sense but the contents are private
                };
                TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
            }
            TransactionVariant::CreateOrderbookTransaction(create_orderbook) => {
                let hasher_update = |hasher: &mut VarBlake2b| {
                    hasher.update(self.get_sender().0.to_bytes());
                    hasher.update(create_orderbook.base_asset_id.to_le_bytes());
                    hasher.update(create_orderbook.quote_asset_id.to_le_bytes());
                    hasher.update(self.get_recent_certificate_digest().to_string());
                };
                TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
            }
            TransactionVariant::PlaceOrderTransaction(order) => match order {
                OrderRequest::Limit {
                    base_asset_id,
                    quote_asset_id,
                    side,
                    price,
                    quantity,
                    local_timestamp,
                } => {
                    let ts = convert_system_time_to_int(*local_timestamp);
                    let hasher_update = |hasher: &mut VarBlake2b| {
                        hasher.update(self.get_sender().0.to_bytes());
                        hasher.update(base_asset_id.to_le_bytes());
                        hasher.update(quote_asset_id.to_le_bytes());
                        hasher.update((*side as u8).to_le_bytes());
                        hasher.update(price.to_le_bytes());
                        hasher.update(quantity.to_le_bytes());
                        hasher.update(ts.to_le_bytes());
                        hasher.update(self.get_recent_certificate_digest().to_string());
                    };
                    TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
                }
                OrderRequest::Market {
                    base_asset_id,
                    quote_asset_id,
                    side,
                    quantity,
                    local_timestamp,
                } => {
                    let ts = convert_system_time_to_int(*local_timestamp);
                    let hasher_update = |hasher: &mut VarBlake2b| {
                        hasher.update(self.get_sender().0.to_bytes());
                        hasher.update(base_asset_id.to_le_bytes());
                        hasher.update(quote_asset_id.to_le_bytes());
                        hasher.update((*side as u8).to_le_bytes());
                        hasher.update(quantity.to_le_bytes());
                        hasher.update(ts.to_le_bytes());
                        hasher.update(self.get_recent_certificate_digest().to_string());
                    };
                    TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
                }
                OrderRequest::CancelOrder { id, side } => {
                    let hasher_update = |hasher: &mut VarBlake2b| {
                        hasher.update(self.get_sender().0.to_bytes());
                        hasher.update(id.to_le_bytes());
                        hasher.update((*side as u8).to_le_bytes());
                        hasher.update(self.get_recent_certificate_digest().to_string());
                    };
                    TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
                }
                OrderRequest::Update {
                    id,
                    side,
                    price,
                    quantity,
                    local_timestamp,
                } => {
                    let ts = convert_system_time_to_int(*local_timestamp);
                    let hasher_update = |hasher: &mut VarBlake2b| {
                        hasher.update(self.get_sender().0.to_bytes());
                        hasher.update(id.to_le_bytes());
                        hasher.update((*side as u8).to_le_bytes());
                        hasher.update(price.to_le_bytes());
                        hasher.update(quantity.to_le_bytes());
                        hasher.update(ts.to_le_bytes());
                        hasher.update(self.get_recent_certificate_digest().to_string());
                    };
                    TransactionDigest(narwhal_crypto::blake2b_256(hasher_update))
                }
            },
        }
    }
}

/// The SignedTransaction object is responsible for encoding
/// a transaction payload and associated metadata which allows
/// validation of sender logic
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedTransaction {
    sender: AccountPubKey,
    transaction_payload: Transaction,
    transaction_signature: AccountSignature,
}

impl SignedTransaction {
    pub fn new(
        sender: AccountPubKey,
        transaction_payload: Transaction,
        transaction_signature: AccountSignature,
    ) -> Self {
        SignedTransaction {
            sender,
            transaction_payload,
            transaction_signature,
        }
    }

    pub fn deserialize(byte_vec: Vec<u8>) -> Result<Self, GDEXError> {
        match bincode::deserialize(&byte_vec[..]) {
            Ok(result) => Ok(result),
            Err(..) => Err(GDEXError::TransactionDeserialization),
        }
    }

    pub fn deserialize_and_verify(byte_vec: Vec<u8>) -> Result<Self, GDEXError> {
        let deserialized_transaction = Self::deserialize(byte_vec)?;
        deserialized_transaction.verify()?;
        Ok(deserialized_transaction)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, GDEXError> {
        match bincode::serialize(&self) {
            Ok(result) => Ok(result),
            Err(..) => Err(GDEXError::TransactionSerialization),
        }
    }

    pub fn get_transaction_payload(&self) -> &Transaction {
        &self.transaction_payload
    }

    pub fn get_transaction_signature(&self) -> &AccountSignature {
        &self.transaction_signature
    }

    pub fn verify(&self) -> Result<(), GDEXError> {
        let transaction_digest_array = self.transaction_payload.digest().get_array();
        match self
            .transaction_payload
            .get_sender()
            .verify(&transaction_digest_array[..], &self.transaction_signature)
        {
            Ok(..) => Ok(()),
            Err(..) => Err(GDEXError::FailedVerification),
        }
    }
}

use crate::account::AccountKeyPair;
use crate::crypto::KeypairTraits;

pub fn create_payment_transaction(
    sender_kp: &AccountKeyPair,
    receiver_kp: &AccountKeyPair,
    asset_id: u64,
    amount: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    let transaction_variant =
        TransactionVariant::PaymentTransaction(PaymentRequest::new(receiver_kp.public().clone(), asset_id, amount));

    Transaction::new(sender_kp.public().clone(), recent_block_hash, transaction_variant)
}

pub fn create_asset_creation_transaction(sender_kp: &AccountKeyPair, recent_block_hash: CertificateDigest) -> Transaction {
    let transaction_variant = TransactionVariant::CreateAssetTransaction(CreateAssetRequest {});

    Transaction::new(sender_kp.public().clone(), recent_block_hash, transaction_variant)
}

pub fn create_orderbook_creation_transaction(
    sender_kp: &AccountKeyPair,
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    let transaction_variant =
        TransactionVariant::CreateOrderbookTransaction(CreateOrderbookRequest::new(base_asset_id, quote_asset_id));

    Transaction::new(sender_kp.public().clone(), recent_block_hash, transaction_variant)
}

pub fn create_place_limit_order_transaction(
    sender_kp: &AccountKeyPair,
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    side: OrderSide,
    price: AssetPrice,
    quantity: AssetAmount,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    let local_timestamp = SystemTime::now();
    let transaction_variant = TransactionVariant::PlaceOrderTransaction(OrderRequest::Limit {
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        local_timestamp,
    });

    Transaction::new(sender_kp.public().clone(), recent_block_hash, transaction_variant)
}

/// Begin externally available testing functions
#[cfg(any(test, feature = "testing"))]
pub mod transaction_test_functions {
    use super::*;
    use crate::{
        account::AccountKeyPair,
        crypto::{KeypairTraits, Signer},
    };

    pub const PRIMARY_ASSET_ID: u64 = 0;

    pub fn generate_signed_test_transaction(
        kp_sender: &AccountKeyPair,
        kp_receiver: &AccountKeyPair,
    ) -> SignedTransaction {
        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);
        let transaction_variant = TransactionVariant::PaymentTransaction(PaymentRequest::new(
            kp_receiver.public().clone(),
            PRIMARY_ASSET_ID,
            10,
        ));

        let transaction = Transaction::new(kp_sender.public().clone(), dummy_batch_digest, transaction_variant);

        let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);

        SignedTransaction::new(kp_sender.public().clone(), transaction, signed_digest)
    }
}

/// Begin the testing suite for transactions
#[cfg(test)]
pub mod transaction_tests {
    use super::transaction_test_functions::*;
    use super::*;
    use crate::account::account_test_functions::generate_keypair_vec;
    use crate::crypto::{KeypairTraits, Signer};

    #[test]
    // test that transaction returns expected fields, validates a good signature, and has deterministic hashing
    fn fails_bad_signature() {
        // generating a signed transaction payload
        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);
        let transaction_variant = TransactionVariant::PaymentTransaction(PaymentRequest::new(
            kp_receiver.public().clone(),
            PRIMARY_ASSET_ID,
            10,
        ));

        let transaction = Transaction::new(kp_sender.public().clone(), dummy_batch_digest, transaction_variant);

        // sign the wrong payload
        let signed_digest = kp_sender.sign(dummy_batch_digest.to_string().as_bytes());

        let signed_transaction = SignedTransaction::new(kp_sender.public().clone(), transaction, signed_digest);
        let verify_result = signed_transaction.verify();

        // check that verification fails
        match verify_result {
            Ok(..) => {
                panic!("An error is expected.");
            }
            Err(GDEXError::FailedVerification) => { /* do nothing */ }
            _ => {
                panic!("An unexpected error occurred.")
            }
        }
    }

    #[test]
    // test that transaction returns expected fields, validates a good signature, and has deterministic hashing
    fn transaction_properties() {
        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);
        let transaction = signed_transaction.get_transaction_payload();
        let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);

        // perform transaction checks

        // check valid signature
        signed_transaction.verify().unwrap();

        // verify deterministic hashing
        let transaction_hash_0 = transaction.digest();
        let transaction_hash_1 = transaction.digest();
        assert!(
            transaction_hash_0 == transaction_hash_1,
            "hashes appears to have violated determinism"
        );
        assert!(
            signed_transaction.get_transaction_signature().clone() == signed_digest,
            "transaction sender does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction_payload().get_sender() == kp_sender.public(),
            "transaction payload sender does not match transction input"
        );
        assert!(
            signed_transaction
                .get_transaction_payload()
                .get_recent_certificate_digest()
                .to_string()
                == CertificateDigest::new([0; DIGEST_LEN]).to_string(),
            "transaction payload batch digest does not match transaction input"
        );
    }

    #[test]
    // test that a signed payment transaction behaves as expected
    fn signed_payment_transaction() {
        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        let payment = match signed_transaction.get_transaction_payload().get_variant() {
            TransactionVariant::PaymentTransaction(r) => r,
            _ => {
                panic!("An unexpected error occurred while reading the payment transaction");
            }
        };

        assert!(
            payment.get_amount() == 10,
            "transaction amount does not match transaction input"
        );
        assert!(
            payment.get_asset_id() == PRIMARY_ASSET_ID,
            "transaction asset id does not match transaction input"
        );
        assert!(
            payment.get_receiver() == kp_receiver.public(),
            "transaction to does not match transction input"
        );
    }

    #[test]
    fn create_asset_transaction() {
        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();

        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);

        let transaction_variant = TransactionVariant::CreateAssetTransaction(CreateAssetRequest {});

        let transaction = Transaction::new(kp_sender.public().clone(), dummy_batch_digest, transaction_variant);
        let signed_digest: AccountSignature = kp_sender.sign(&transaction.digest().get_array()[..]);

        let signed_transaction =
            SignedTransaction::new(kp_sender.public().clone(), transaction.clone(), signed_digest.clone());

        // check valid signature
        signed_transaction.verify().unwrap();

        // check we can unpack transaction as expected
        let _create_asset = match signed_transaction.get_transaction_payload().get_variant() {
            TransactionVariant::CreateAssetTransaction(r) => r,
            _ => {
                panic!("An unexpected error occurred while reading the payment transaction");
            }
        };
    }

    #[test]
    fn test_serialize_deserialize() {
        let kp_sender = generate_keypair_vec([0; 32]).pop().unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();
        let signed_transaction = generate_signed_test_transaction(&kp_sender, &kp_receiver);

        // perform transaction checks

        let serialized = signed_transaction.serialize().unwrap();
        // check valid signature
        let signed_transaction_deserialized: SignedTransaction = SignedTransaction::deserialize(serialized).unwrap();

        // verify signed transaction matches previous values
        assert!(
            signed_transaction.get_transaction_signature()
                == signed_transaction_deserialized.get_transaction_signature(),
            "signed transaction signature does not match after deserialize"
        );

        // verify transaction matches previous values
        let transaction = signed_transaction.get_transaction_payload();
        let transaction_deserialized = signed_transaction_deserialized.get_transaction_payload();

        assert!(
            transaction.digest() == transaction_deserialized.digest(),
            "transaction hash does not match after deserialize"
        );

        assert!(
            transaction.get_sender() == transaction_deserialized.get_sender(),
            "transaction hash does not match"
        );

        // verify transaction variant matches previous values
        let payment = match transaction.get_variant() {
            TransactionVariant::PaymentTransaction(r) => r,
            _ => {
                panic!("An unexpected error occurred while reading the payment transaction");
            }
        };
        let payment_deserialized = match transaction_deserialized.get_variant() {
            TransactionVariant::PaymentTransaction(r) => r,
            _ => {
                panic!("An unexpected error occurred while reading the payment transaction");
            }
        };

        assert!(
            payment_deserialized.get_amount() == payment.get_amount(),
            "transaction amount does not match transaction input"
        );

        assert!(
            payment_deserialized.get_asset_id() == payment.get_asset_id(),
            "transaction amount does not match transaction input"
        );

        assert!(
            payment_deserialized.get_receiver() == payment.get_receiver(),
            "transaction amount does not match transaction input"
        );
    }
}
