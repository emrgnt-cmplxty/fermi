extern crate rocksdb;

extern crate core;
extern crate engine;
extern crate types;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};
use rocksdb::{ColumnFamilyDescriptor, DB, DBWithThreadMode, Options, SingleThreaded};
use std::time::SystemTime;

use core::{
    transaction::{
        Order, 
        TxnRequest, 
        TxnVariant,
    }
};
use diem_crypto::{
    traits::{Uniform, SigningKey},
    hash::{CryptoHash, HashValue},
};
use engine::{
    orders::{new_limit_order_request},
    orderbook::{Orderbook}

};
use proc::{
    bank::{BankController},
    spot::{SpotController, DUMMY_MESSAGE}
};
use types::{
    account::{
        AccountPubKey, 
        AccountPrivKey, 
        AccountSignature
    },
    orderbook::{
        OrderProcessingResult, 
        OrderSide, 
        Success
    },
    spot::{TestDiemCrypto},
};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;
const BASE_ASSET_ID: u64 = 0;
const QUOTE_ASSET_ID: u64 = 1;
const TRANSFER_AMOUNT: u64 = 500_000_000;

fn persist_result(db: &DBWithThreadMode<SingleThreaded>, proc_result: &OrderProcessingResult) -> () {
    for result in proc_result {
        match result {
            Ok(Success::Accepted { order_id, .. }) => {
                db.put(order_id.to_string(), "a").unwrap();
                order_id
            },
            Ok(Success::PartiallyFilled { order_id, .. }) => {
                db.put(order_id.to_string(), "pf").unwrap();
                order_id
            },
            Ok(Success::Filled { order_id, .. }) => {
                db.put(order_id.to_string(), "f").unwrap();
                order_id
            },
            _ => &0
        };
    }
}

fn whole_shebang(
    n_orders: u64, 
    signed_txns: &mut Vec<TxnRequest<TxnVariant>>, 
    bank_controller: &mut BankController, 
    spot_controller: &mut SpotController, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let orderbook: Orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

    // clean market controller orderbook to keep from getting clogged
    spot_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    for txn in signed_txns {
        let res: Vec<Result<Success, types::orderbook::Failed>> = spot_controller.parse_limit_order_txn(bank_controller, txn).unwrap();
        // if persist {
        //     persist_result(db, &res);
        // }
        i_order+=1;
    }
}
    
pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize market controller
    let mut account_to_pub_key: Vec<AccountPubKey> = Vec::new();
    let mut account_to_signed_msg: Vec<AccountSignature> = Vec::new();
    let mut bank_controller: BankController = BankController::new();
    let mut spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID);
    // other helpers
    let mut rng: ThreadRng = rand::thread_rng();
    let mut i_account: u64 = 0;
    let path: &str = "./db.rocks";
    let mut cf_opts: Options = Options::default();
    cf_opts.set_max_write_buffer_number(16);
    let cf: ColumnFamilyDescriptor = ColumnFamilyDescriptor::new("cf1", cf_opts);
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);
    let db: DBWithThreadMode<SingleThreaded> = DB::open_cf_descriptors(&db_opts, path, vec![cf]).unwrap();

    // instantiating joint vector
    let mut keys_and_signatures: Vec<(AccountPubKey, AccountSignature)> = Vec::new();
    
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let creator_key: AccountPubKey = (&private_key).into();
    // create spot & bank accounts and create assets for test
    spot_controller.create_account(&creator_key).unwrap();
    bank_controller.create_account(&creator_key).unwrap();
    bank_controller.create_asset(&creator_key);
    bank_controller.create_asset(&creator_key);
    // generate N_ACCOUNTS accounts to transact w/ orderbook
    while i_account < N_ACCOUNTS - 1 {
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        spot_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.transfer(&creator_key, &account_pub_key, BASE_ASSET_ID, TRANSFER_AMOUNT);
        bank_controller.transfer(&creator_key, &account_pub_key, QUOTE_ASSET_ID, TRANSFER_AMOUNT);

        // create initial asset
        account_to_pub_key.push(account_pub_key);
        let sig: AccountSignature  = private_key.sign(&TestDiemCrypto(DUMMY_MESSAGE.to_string()));
        account_to_signed_msg.push(sig.clone());

        i_account += 1;
        keys_and_signatures.push((account_pub_key, sig));
    }
    let mut signed_txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

    let mut i_order: u64 = 0;
    while i_order < N_ORDERS_BENCH {

        let order_type: OrderSide = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);
        // generate a random integer between 0 and 100
        let account_pub_key: AccountPubKey = account_to_pub_key[rng.gen_range(1, 100)];
        let txn: TxnVariant = TxnVariant::OrderTransaction(
            Order{
                request: 
                    new_limit_order_request(
                        BASE_ASSET_ID,
                        QUOTE_ASSET_ID,
                        order_type,
                        price,
                        qty,
                        SystemTime::now()
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
        signed_txns.push(signed_txn);
        i_order += 1;
    }

    c.bench_function("whole_shebang", |b| b.iter(|| 
        whole_shebang(N_ORDERS_BENCH, &mut signed_txns, &mut bank_controller, &mut spot_controller, &mut rng, &db, true)));

}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
