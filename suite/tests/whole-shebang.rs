extern crate engine;
extern crate core;

use rand::rngs::{ThreadRng};
use std::{time};

use core::{
    transaction::{
        Order, 
        TxnRequest, 
        TxnVariant,
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
use proc::{
    bank::{BankController},
    spot::{SpotController}
};
use types::{
    account::{AccountPubKey, AccountPrivKey, AccountSignature},
    orderbook::{OrderSide},
    spot::{TestDiemCrypto}
};


#[test]
fn test_orderbook_signed_txn() {
    let mut rng: ThreadRng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let account_pub_key: AccountPubKey = (&private_key).into();

    let mut bank_controller: BankController = BankController::new();
    bank_controller.create_account(&account_pub_key).unwrap();
    let base_asset_id = bank_controller.create_asset(&account_pub_key);
    let quote_asset_id = bank_controller.create_asset(&account_pub_key);

    let mut spot_controller: SpotController = SpotController::new(base_asset_id, quote_asset_id);
    spot_controller.create_account(&account_pub_key).unwrap();


    let price = 1;
    let qty = 10;
    let txn: TxnVariant = TxnVariant::OrderTransaction(
        Order {
            request: 
            new_limit_order_request(
                base_asset_id,
                quote_asset_id,
                OrderSide::Bid,
                price,
                qty,
                time::SystemTime::now()
            )
        }
    );

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&TestDiemCrypto(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>{
        txn: txn,
        sender_address: account_pub_key, 
        txn_signature: signed_hash 
    };

    spot_controller.parse_limit_order_txn(&mut bank_controller, &signed_txn).unwrap();
    verify_transaction::<TxnVariant>(&signed_txn, &account_pub_key); 

    let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();
    txns.push(signed_txn);
    let block_hash: HashValue = generate_block_hash(&txns);
    let block: Block<TxnVariant> = Block::<TxnVariant>{ txns: txns, block_hash: block_hash};
    let mut blocks: Vec<Block<TxnVariant>> = Vec::new();
    blocks.push(block);
    BlockContainer{blocks};

}
