mod test {
    extern crate core;
    extern crate engine;

    use core::{
        block::{generate_block_hash, Block, BlockContainer},
        hash_clock::HashClock,
        transaction::{
            CreateAssetRequest, OrderRequest, PaymentRequest, TransactionRequest, TransactionVariant,
            TransactionVariant::CreateAssetTransaction, TransactionVariant::OrderTransaction,
            TransactionVariant::PaymentTransaction,
        },
        vote_cert::VoteCert,
    };
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
        let transaction: OrderRequest = new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            quantity,
            time::SystemTime::now(),
        );

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<OrderRequest> =
            TransactionRequest::<OrderRequest>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
    }

    #[test]
    fn create_signed_payment_transaction() {
        let (sender_pub_key, private_key) = generate_key_pair();
        let (receiver_pub_key, _receiver_private_key) = generate_key_pair();

        let transaction: PaymentRequest = PaymentRequest::new(sender_pub_key, receiver_pub_key, BASE_ASSET_ID, 10);

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<PaymentRequest> =
            TransactionRequest::<PaymentRequest>::new(transaction, sender_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
    }

    #[test]
    fn create_asset_transaction() {
        let (sender_pub_key, private_key) = generate_key_pair();

        let transaction: CreateAssetRequest = CreateAssetRequest {};

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<CreateAssetRequest> =
            TransactionRequest::<CreateAssetRequest>::new(transaction, sender_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
    }

    #[test]
    fn block_hash_test() {
        let (account_pub_key, private_key) = generate_key_pair();
        let (receiver_pub_key, _receiver_private_key) = generate_key_pair();

        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();

        let price: u64 = 1;
        let quantity: u64 = 10;
        let transaction: TransactionVariant = OrderTransaction(new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            quantity,
            time::SystemTime::now(),
        ));
        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<TransactionVariant> =
            TransactionRequest::<TransactionVariant>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
        transactions.push(signed_transaction);

        let transaction: TransactionVariant = PaymentTransaction(PaymentRequest::new(
            account_pub_key,
            receiver_pub_key,
            BASE_ASSET_ID,
            10,
        ));
        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<TransactionVariant> =
            TransactionRequest::<TransactionVariant>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
        transactions.push(signed_transaction);

        let transaction: TransactionVariant = CreateAssetTransaction(CreateAssetRequest {});
        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<TransactionVariant> =
            TransactionRequest::<TransactionVariant>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
        transactions.push(signed_transaction);

        let block_hash: HashValue = generate_block_hash(&transactions);
        let hash_clock: HashClock = HashClock::default();
        let dummy_vote_cert: VoteCert = VoteCert::new(0, block_hash);

        let block: Block<TransactionVariant> = Block::<TransactionVariant>::new(
            transactions,
            account_pub_key,
            block_hash,
            0,
            hash_clock.get_hash_time(),
            dummy_vote_cert,
        );

        let mut block_container: BlockContainer<TransactionVariant> = BlockContainer::new();
        block_container.append_block(block);
    }
}
