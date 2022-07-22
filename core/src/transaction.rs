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
use std::{
    fmt::Debug,
    sync::{Arc, Mutex, MutexGuard},
    thread::{spawn, JoinHandle},
    time::SystemTime,
};
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
pub mod batch_functions {
    use super::*;

    // leverage dalek batch transaction calculation speedup
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

    fn get_verification_handler(
        transaction_requests: Vec<TransactionRequest<TransactionVariant>>,
    ) -> JoinHandle<Result<(), gdex_crypto::error::Error>> {
        let transaction_requests = Arc::new(Mutex::new(transaction_requests));
        spawn(move || {
            let transaction_requests: MutexGuard<Vec<TransactionRequest<TransactionVariant>>> =
                transaction_requests.lock().unwrap();
            verify_transaction_batch_multithread(transaction_requests)
        })
    }

    // combine batch transactions with multithreading
    fn verify_transaction_batch_multithread(
        transaction_requests: MutexGuard<Vec<TransactionRequest<TransactionVariant>>>,
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

    pub fn verify_transaction_batch_multithreaded(
        transaction_requests: Vec<TransactionRequest<TransactionVariant>>,
        n_threads: u64,
    ) -> Result<(), gdex_crypto::error::Error> {
        let mut transaction_handlers: Vec<JoinHandle<Result<(), gdex_crypto::error::Error>>> = Vec::new();
        // use chunk to evenly split and spawn processes over the allotted threads
        for chunk in transaction_requests.chunks(transaction_requests.len() / (n_threads as usize)) {
            let transaction_handler: JoinHandle<Result<(), gdex_crypto::error::Error>> =
                get_verification_handler(chunk.to_vec());
            transaction_handlers.push(transaction_handler);
        }
        // verify success in turn
        for transaction_handler in transaction_handlers.into_iter() {
            transaction_handler.join().unwrap()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::transaction::{CreateAssetRequest, PaymentRequest, TransactionRequest};
    use gdex_crypto::{
        hash::{CryptoHash, HashValue},
        SigningKey, Uniform,
    };
    use types::account::{AccountPrivKey, AccountSignature};

    const PRIMARY_ASSET_ID: u64 = 0;

    #[test]
    fn create_signed_payment_transaction() {
        let private_key: AccountPrivKey = AccountPrivKey::generate_for_testing(0);
        let sender_pub_key: AccountPubKey = (&private_key).into();

        let receiver_private_key: AccountPrivKey = AccountPrivKey::generate_for_testing(1);
        let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();

        let transaction: PaymentRequest = PaymentRequest::new(sender_pub_key, receiver_pub_key, PRIMARY_ASSET_ID, 10);

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<PaymentRequest> =
            TransactionRequest::<PaymentRequest>::new(transaction, sender_pub_key, signed_hash.clone());
        signed_transaction.verify_transaction().unwrap();

        assert!(
            signed_transaction.get_sender().clone() == sender_pub_key,
            "transaction sender does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction_signature().clone() == signed_hash,
            "transaction sender does not match transaction input"
        );

        assert!(
            signed_transaction.get_transaction().get_amount() == 10,
            "transaction amount does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction().get_asset_id() == PRIMARY_ASSET_ID,
            "transaction asset id does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction().get_from().clone() == sender_pub_key,
            "transaction from does not match transction input"
        );
        assert!(
            signed_transaction.get_transaction().get_to().clone() == receiver_pub_key,
            "transaction to does not match transction input"
        );
    }

    #[test]
    fn create_signed_stake_transaction() {
        let private_key: AccountPrivKey = AccountPrivKey::generate_for_testing(0);
        let sender_pub_key: AccountPubKey = (&private_key).into();

        let transaction: StakeRequest = StakeRequest::new(sender_pub_key, 10);

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<StakeRequest> =
            TransactionRequest::<StakeRequest>::new(transaction, sender_pub_key, signed_hash.clone());
        signed_transaction.verify_transaction().unwrap();

        assert!(
            signed_transaction.get_sender().clone() == sender_pub_key,
            "transaction sender does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction_signature().clone() == signed_hash,
            "transaction signature does not match transaction input"
        );

        assert!(
            signed_transaction.get_transaction().get_amount() == 10,
            "transaction amount does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction().get_from().clone() == sender_pub_key,
            "transaction from does not match transction input"
        );
    }

    #[test]
    fn create_asset_transaction() {
        let private_key: AccountPrivKey = AccountPrivKey::generate_for_testing(0);
        let sender_pub_key: AccountPubKey = (&private_key).into();

        let transaction: CreateAssetRequest = CreateAssetRequest {};

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<CreateAssetRequest> =
            TransactionRequest::<CreateAssetRequest>::new(transaction, sender_pub_key, signed_hash.clone());
        signed_transaction.verify_transaction().unwrap();

        assert!(
            signed_transaction.get_sender().clone() == sender_pub_key,
            "transaction sender does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction_signature().clone() == signed_hash,
            "transaction signature does not match transaction input"
        );
    }

    #[test]
    fn create_orderbook_transaction() {
        let private_key: AccountPrivKey = AccountPrivKey::generate_for_testing(0);
        let sender_pub_key: AccountPubKey = (&private_key).into();
        let dummy_asset_id = 1;

        let transaction: CreateOrderbookRequest = CreateOrderbookRequest::new(PRIMARY_ASSET_ID, dummy_asset_id);

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<CreateOrderbookRequest> =
            TransactionRequest::<CreateOrderbookRequest>::new(transaction, sender_pub_key, signed_hash.clone());
        signed_transaction.verify_transaction().unwrap();

        assert!(
            signed_transaction.get_sender().clone() == sender_pub_key,
            "transaction sender does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction_signature().clone() == signed_hash,
            "transaction signature does not match transaction input"
        );

        assert!(
            signed_transaction.get_transaction().get_base_asset_id() == PRIMARY_ASSET_ID,
            "transaction base asset does not match transaction input"
        );
        assert!(
            signed_transaction.get_transaction().get_quote_asset_id() == dummy_asset_id,
            "transaction quote asset does not match transction input"
        );
    }
}
