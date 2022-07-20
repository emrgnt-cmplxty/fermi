//! THE WHOLE SHEBANG TEST IS AN ONGOING WORK IN PROGRESS
//! HERE WE WILL CONSTRUCT A LARGE-SCALE TEST THAT LOOKS
//! TO SHOW THAT THE ENTIRE CODEBASE IS GENERALLY SOUND
//! 
extern crate engine;
extern crate core;

use rand::rngs::{ThreadRng};
use std::{time};

use core::{
    transaction::{
        Order, 
        TxnRequest, 
        TxnVariant,
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
use gdex_crypto::{
    SigningKey,
    Uniform,
    hash::{CryptoHash, HashValue},
};
use engine::orders::{new_limit_order_request};
use proc::{
    bank::{BankController},
    spot::{SpotController},
};
use types::{
    account::{AccountPubKey, AccountPrivKey, AccountSignature},
    orderbook::{OrderSide},
    spot::{DiemCryptoMessage},
};


#[test]
fn test_orderbook_signed_txn() {
    let mut rng: ThreadRng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let account_pub_key: AccountPubKey = (&private_key).into();

    let mut bank_controller: BankController = BankController::new();
    let base_asset_id = bank_controller.create_asset(&account_pub_key).unwrap();
    let quote_asset_id = bank_controller.create_asset(&account_pub_key).unwrap();

    let mut spot_controller: SpotController = SpotController::new(base_asset_id, quote_asset_id);


    let price = 1;
    let qty = 10;
    let txn: TxnVariant = TxnVariant::OrderTransaction(
        Order::new(
            new_limit_order_request(
                base_asset_id,
                quote_asset_id,
                OrderSide::Bid,
                price,
                qty,
                time::SystemTime::now()
            )
        )
    );

    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
    let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(
        txn,
        account_pub_key, 
        signed_hash 
    );

    spot_controller.parse_limit_order_txn(&mut bank_controller, &signed_txn).unwrap();
    signed_txn.verify_transaction().unwrap();

    let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();
    txns.push(signed_txn);
    let block_hash: HashValue = generate_block_hash(&txns);
    let hash_clock: HashClock = HashClock::new();
    
    let dummy_vote_cert: VoteCert = VoteCert::new(0, block_hash);
    let block: Block<TxnVariant> = Block::<TxnVariant>::new(txns, account_pub_key, block_hash, hash_clock.get_time(), dummy_vote_cert);

    let mut block_container:BlockContainer<TxnVariant> = BlockContainer::new();
    block_container.append_block(block);
}
