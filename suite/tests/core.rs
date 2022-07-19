extern crate engine;
extern crate core;

use std::{time};
use core::{
    transaction::{
        CreateAsset,
        Order, 
        Payment,
        TxnRequest, 
        TxnVariant,
        TxnVariant::PaymentTransaction,
        TxnVariant::OrderTransaction,
        TxnVariant::CreateAssetTransaction,
        verify_transaction,
    },
    block::{
        Block, 
        BlockContainer, 
        generate_block_hash
    },
    hash_clock::{
        HashClock
    },
    vote_cert::{
        VoteCert
    },
};
use diem_crypto::{
    SigningKey,
    Uniform,
    hash::{CryptoHash, HashValue},
};
use engine::orders::{new_limit_order_request};
use types::{
    account::{AccountPubKey, AccountPrivKey, AccountSignature},
    orderbook::{OrderSide},
    spot::{DiemCryptoMessage}
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
    let txn: Order = Order::new(
        new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            qty,
            time::SystemTime::now()
        ),
    );

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<Order> = TxnRequest::<Order>::new(
        txn,
        account_pub_key, 
        signed_hash 
    );
    verify_transaction::<Order>(&signed_txn, &account_pub_key); 
}

#[test]
fn create_signed_payment_transaction() {
    let mut rng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let sender_pub_key: AccountPubKey = (&private_key).into();
    let receiver_private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();
    
    let txn: Payment = Payment::new(
        sender_pub_key,
        receiver_pub_key,
        BASE_ASSET_ID,
        10
    );

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<Payment> = TxnRequest::<Payment>::new(
        txn,
        sender_pub_key, 
        signed_hash 
    );
    verify_transaction::<Payment>(&signed_txn, &sender_pub_key); 
}

#[test]
fn create_asset_transaction() {
    let mut rng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let sender_pub_key: AccountPubKey = (&private_key).into();
    
    let txn: CreateAsset = CreateAsset{};

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<CreateAsset> = TxnRequest::<CreateAsset>::new(
        txn,
        sender_pub_key, 
        signed_hash 
    );
    verify_transaction::<CreateAsset>(&signed_txn, &sender_pub_key); 
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
    let txn: TxnVariant = OrderTransaction(
        Order::new(
            new_limit_order_request(
                BASE_ASSET_ID,
                QUOTE_ASSET_ID,
                OrderSide::Bid,
                price,
                qty,
                time::SystemTime::now()
            ),
        )
    );
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant>= TxnRequest::<TxnVariant>::new(
        txn,
        account_pub_key, 
        signed_hash,
    );
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key);
    txns.push(signed_txn);

    let txn: TxnVariant = PaymentTransaction(
        Payment::new(
            account_pub_key,
            receiver_pub_key,
            BASE_ASSET_ID,
            10,
        )
    );
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(
        txn,
        account_pub_key, 
        signed_hash, 
    );
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key); 
    txns.push(signed_txn);


    let txn: TxnVariant = CreateAssetTransaction(CreateAsset{});
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(
        txn,
        account_pub_key, 
        signed_hash, 
    );
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key); 
    txns.push(signed_txn);

    let block_hash: HashValue = generate_block_hash(&txns);
    let hash_clock: HashClock = HashClock::new();
    let dummy_vote_cert: VoteCert = VoteCert::new(0);

    let block: Block<TxnVariant> = Block::<TxnVariant>::new(txns, account_pub_key, block_hash, hash_clock.get_time(), dummy_vote_cert);
    
    let mut block_container:BlockContainer<TxnVariant> = BlockContainer::new();
    block_container.append_block(block);
}