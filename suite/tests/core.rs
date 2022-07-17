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
    }
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
    spot::{TestDiemCrypto}
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
    let txn: Order = Order{
        request: 
        new_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            price,
            qty,
            time::SystemTime::now()
        )
    };

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<Order> = TxnRequest::<Order>{
        txn: txn,
        sender_address: account_pub_key, 
        txn_signature: signed_hash 
    };
    verify_transaction::<Order>(&signed_txn, &account_pub_key); 
}

#[test]
fn create_signed_payment_transaction() {
    let mut rng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let sender_pub_key: AccountPubKey = (&private_key).into();
    let receiver_private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();
    
    let txn: Payment = Payment{
        from: sender_pub_key,
        to: receiver_pub_key,
        asset_id: BASE_ASSET_ID,
        amount: 10
    };

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<Payment> = TxnRequest::<Payment>{
        txn,
        sender_address: sender_pub_key, 
        txn_signature: signed_hash 
    };
    verify_transaction::<Payment>(&signed_txn, &sender_pub_key); 
}

#[test]
fn create_asset_transaction() {
    let mut rng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let sender_pub_key: AccountPubKey = (&private_key).into();
    
    let txn: CreateAsset = CreateAsset{};

    let txn_hash = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<CreateAsset> = TxnRequest::<CreateAsset>{
        txn,
        sender_address: sender_pub_key, 
        txn_signature: signed_hash 
    };
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
    
    let price = 1;
    let qty = 10;
    let txn: TxnVariant = OrderTransaction(
        Order{
            request: 
                new_limit_order_request(
                    BASE_ASSET_ID,
                    QUOTE_ASSET_ID,
                    OrderSide::Bid,
                    price,
                    qty,
                    time::SystemTime::now()
                )
        }
    );
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant>= TxnRequest::<TxnVariant>{
        txn: txn,
        sender_address: account_pub_key, 
        txn_signature: signed_hash 
    };
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key);
    txns.push(signed_txn);

    let txn: TxnVariant = PaymentTransaction(
        Payment{
            from: account_pub_key,
            to: receiver_pub_key,
            asset_id: BASE_ASSET_ID,
            amount: 10
        }
    );
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>{
        txn,
        sender_address: account_pub_key, 
        txn_signature: signed_hash 
    };
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key); 
    txns.push(signed_txn);


    let txn: TxnVariant = CreateAssetTransaction(CreateAsset{});
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>{
        txn,
        sender_address: account_pub_key, 
        txn_signature: signed_hash 
    };
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key); 
    txns.push(signed_txn);

    println!("txn_hash={}", txn_hash);
    let block_hash: HashValue = generate_block_hash(&txns);
    println!("block_hash={}", block_hash);
    let block: Block<TxnVariant> = Block::<TxnVariant>{ txns: txns, block_hash: block_hash};
    let mut blocks: Vec<Block<TxnVariant>> = Vec::new();
    blocks.push(block);
    BlockContainer{blocks};
}