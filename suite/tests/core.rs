mod test {
    extern crate core;
    extern crate engine;

    use core::{
        block::{generate_block_hash, Block, BlockContainer},
        hash_clock::HashClock,
        transaction::{
            CreateAssetRequest, OrderRequest, PaymentRequest, TxnRequest, TxnVariant,
            TxnVariant::CreateAssetTransaction, TxnVariant::OrderTransaction, TxnVariant::PaymentTransaction,
        },
        vote_cert::VoteCert,
    };
    use engine::orders::new_limit_order_request;
    use gdex_crypto::{
        hash::{CryptoHash, HashValue},
        SigningKey, Uniform,
    };
    use std::time;
    use types::{
        account::{AccountPrivKey, AccountPubKey, AccountSignature},
        orderbook::OrderSide,
        spot::DiemCryptoMessage,
    };

    const BASE_ASSET_ID: u64 = 0;
    const QUOTE_ASSET_ID: u64 = 1;

    #[test]
    fn create_signed_order_transaction() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let price = 1;
        let qty = 10;
        let txn: OrderRequest = new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            qty,
            time::SystemTime::now(),
        );

        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<OrderRequest> = TxnRequest::<OrderRequest>::new(txn, account_pub_key, signed_hash);
        signed_txn.verify_transaction().unwrap();
    }

    #[test]
    fn create_signed_payment_transaction() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let sender_pub_key: AccountPubKey = (&private_key).into();
        let receiver_private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();

        let txn: PaymentRequest = PaymentRequest::new(sender_pub_key, receiver_pub_key, BASE_ASSET_ID, 10);

        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<PaymentRequest> =
            TxnRequest::<PaymentRequest>::new(txn, sender_pub_key, signed_hash);
        signed_txn.verify_transaction().unwrap();
    }

    #[test]
    fn create_asset_transaction() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let sender_pub_key: AccountPubKey = (&private_key).into();

        let txn: CreateAssetRequest = CreateAssetRequest {};

        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<CreateAssetRequest> =
            TxnRequest::<CreateAssetRequest>::new(txn, sender_pub_key, signed_hash);
        signed_txn.verify_transaction().unwrap();
    }

    #[test]
    fn block_hash_test() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();
        let receiver_private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();

        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

        let price: u64 = 1;
        let qty: u64 = 10;
        let txn: TxnVariant = OrderTransaction(new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            qty,
            time::SystemTime::now(),
        ));
        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(txn, account_pub_key, signed_hash);
        signed_txn.verify_transaction().unwrap();
        txns.push(signed_txn);

        let txn: TxnVariant = PaymentTransaction(PaymentRequest::new(
            account_pub_key,
            receiver_pub_key,
            BASE_ASSET_ID,
            10,
        ));
        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(txn, account_pub_key, signed_hash);
        signed_txn.verify_transaction().unwrap();
        txns.push(signed_txn);

        let txn: TxnVariant = CreateAssetTransaction(CreateAssetRequest {});
        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(txn, account_pub_key, signed_hash);
        signed_txn.verify_transaction().unwrap();
        txns.push(signed_txn);

        let block_hash: HashValue = generate_block_hash(&txns);
        let hash_clock: HashClock = HashClock::new();
        let dummy_vote_cert: VoteCert = VoteCert::new(0, block_hash);

        let block: Block<TxnVariant> = Block::<TxnVariant>::new(
            txns,
            account_pub_key,
            block_hash,
            hash_clock.get_time(),
            dummy_vote_cert,
        );

        let mut block_container: BlockContainer<TxnVariant> = BlockContainer::new();
        block_container.append_block(block);
    }
}
