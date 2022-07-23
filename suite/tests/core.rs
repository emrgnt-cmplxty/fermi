mod test {
    extern crate core;
    extern crate engine;

    use core::transaction::{CreateAssetRequest, OrderRequest, PaymentRequest, TransactionRequest};
    use engine::orders::new_limit_order_request;
    use gdex_crypto::{
        hash::{CryptoHash, HashValue},
        SigningKey,
    };
    use proc::account::generate_key_pair;
    use std::time;
    use types::{account::AccountSignature, orderbook::OrderSide, spot::DiemCryptoMessage};

    const BASE_ASSET_ID: u64 = 0;
    const QUOTE_ASSET_ID: u64 = 1;

    #[test]
    fn create_signed_order_transaction() {
        let (account_pub_key, private_key) = generate_key_pair();

        let price = 1;
        let quantity = 10;
        let transaction = new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            quantity,
            time::SystemTime::now(),
        );

        let transaction_hash = transaction.hash();
        let signed_hash = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction = TransactionRequest::<OrderRequest>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
    }

    #[test]
    fn create_signed_payment_transaction() {
        let (sender_pub_key, private_key) = generate_key_pair();
        let (receiver_pub_key, _receiver_private_key) = generate_key_pair();

        let transaction = PaymentRequest::new(sender_pub_key, receiver_pub_key, BASE_ASSET_ID, 10);

        let transaction_hash = transaction.hash();
        let signed_hash = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction =
            TransactionRequest::<PaymentRequest>::new(transaction, sender_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
    }

    #[test]
    fn create_asset_transaction() {
        let (sender_pub_key, private_key) = generate_key_pair();

        let transaction = CreateAssetRequest {};

        let transaction_hash = transaction.hash();
        let signed_hash = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction =
            TransactionRequest::<CreateAssetRequest>::new(transaction, sender_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
    }
}
