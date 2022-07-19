extern crate rocksdb;

extern crate engine;
extern crate types;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};
use rocksdb::{ColumnFamilyDescriptor, DB, DBWithThreadMode, Options, SingleThreaded};
use std::time::SystemTime;

use diem_crypto::{
    traits::{Uniform, Signature, SigningKey},
};
use engine::{
    orders,
    orderbook::{Orderbook}
};
use proc::{
    bank::{BankController},
    spot::{SpotController, DUMMY_MESSAGE}
};
use types::{
    account::{AccountPubKey, AccountPrivKey, AccountSignature},
    orderbook::{OrderProcessingResult, OrderSide, Success},
    spot::{DiemCryptoMessage}
};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;
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

fn place_orders_engine(
    base_asset_id: u64,
    quote_asset_id: u64,
    n_orders: u64, 
    rng: &mut ThreadRng, 
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let mut orderbook: Orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let qty = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);

        // order construction & submission
        let order = orders::new_limit_order_request(
            base_asset_id,
            quote_asset_id,
            order_type,
            price,
            qty,
            SystemTime::now()
        );
        let res = orderbook.process_order(order);
        if persist {
            persist_result(db, &res);
        }
        i_order+=1;
    }
}

fn place_orders_engine_account(
    n_orders: u64, 
    base_asset_id: u64,
    quote_asset_id: u64,
    account_to_pub_key: &mut Vec<AccountPubKey>, 
    bank_controller: &mut BankController,
    spot_controller: &mut SpotController, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let orderbook: Orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // clean market controller orderbook to keep from getting clogged
    spot_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type: OrderSide = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);
        // generate a random integer between 0 and 100
        let account_pub_key: AccountPubKey = account_to_pub_key[rng.gen_range(1, 100)];

        let res = spot_controller.place_limit_order(bank_controller, &account_pub_key,  order_type, qty, price).unwrap();
        if persist {
            persist_result(db, &res);
        }
        i_order+=1;
    }
}
    
fn place_orders_engine_account_signed(
    n_orders: u64, 
    base_asset_id: u64,
    quote_asset_id: u64,
    account_to_pub_key: &mut Vec<AccountPubKey>, 
    account_to_signed_msg: &mut Vec<AccountSignature>, 
    bank_controller: &mut BankController, 
    spot_controller: &mut SpotController, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let orderbook: Orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // clean market controller orderbook to keep from getting clogged
    spot_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type: OrderSide = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);
        // generate a random integer between 0 and 100
        let i_account = rng.gen_range(0, 100);
        let account_pub_key: AccountPubKey = account_to_pub_key[i_account];
        let account_sig_msg: &AccountSignature = &account_to_signed_msg[i_account];

        let res: Vec<Result<Success, types::orderbook::Failed>>  = spot_controller.place_signed_limit_order(bank_controller, &account_pub_key,  order_type, qty, price, account_sig_msg).unwrap();

        i_order+=1;

        if persist {
            persist_result(db, &res);
        }
    }
}

#[cfg(feature = "batch")]
fn place_orders_engine_account_batch_signed(
    n_orders: u64, 
    base_asset_id: u64,
    quote_asset_id: u64,
    account_to_pub_key: &mut Vec<AccountPubKey>, 
    keys_and_signatures: &mut Vec<(AccountPubKey, AccountSignature)>, 
    bank_controller: &mut BankController, 
    spot_controller: &mut SpotController, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    Signature::batch_verify(&DiemCryptoMessage(DUMMY_MESSAGE.to_string()), keys_and_signatures.to_vec()).unwrap();
    place_orders_engine_account(n_orders, base_asset_id, quote_asset_id, account_to_pub_key, bank_controller, spot_controller, rng, db, persist);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // generate creator details
    let mut rng: ThreadRng = rand::thread_rng();
    let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
    let creator_key: AccountPubKey = (&private_key).into();

    // initialize market controller
    let mut account_to_pub_key: Vec<AccountPubKey> = Vec::new();
    let mut account_to_signed_msg: Vec<AccountSignature> = Vec::new();
    let mut bank_controller: BankController = BankController::new();
    bank_controller.create_account(&creator_key).unwrap();
    let base_asset_id = bank_controller.create_asset(&creator_key);
    let quote_asset_id = bank_controller.create_asset(&creator_key);

    let mut spot_controller: SpotController = SpotController::new(base_asset_id, quote_asset_id);
    spot_controller.create_account(&creator_key).unwrap();
    
    // other helpers
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
    
    // generate N_ACCOUNTS accounts to transact w/ orderbook
    while i_account < N_ACCOUNTS - 1 {
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        spot_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.transfer(&creator_key, &account_pub_key, base_asset_id, TRANSFER_AMOUNT);
        bank_controller.transfer(&creator_key, &account_pub_key, quote_asset_id, TRANSFER_AMOUNT);

        // create initial asset
        account_to_pub_key.push(account_pub_key);
        let sig: AccountSignature  = private_key.sign(&DiemCryptoMessage(DUMMY_MESSAGE.to_string()));
        account_to_signed_msg.push(sig.clone());

        i_account += 1;
        keys_and_signatures.push((account_pub_key, sig));
    }
    

    c.bench_function("place_orders_engine", |b| b.iter(|| 
        place_orders_engine(base_asset_id, quote_asset_id, black_box(N_ORDERS_BENCH), &mut rng, &db, false)));

    c.bench_function("place_orders_engine_db", |b| b.iter(|| 
        place_orders_engine(base_asset_id, quote_asset_id, black_box(N_ORDERS_BENCH), &mut rng, &db, true)));
    
    c.bench_function("place_orders_engine_account", |b| b.iter(|| 
        place_orders_engine_account(black_box(N_ORDERS_BENCH), base_asset_id, quote_asset_id, &mut account_to_pub_key, &mut bank_controller, &mut spot_controller, &mut rng, &db, false)));
    
    c.bench_function("place_orders_engine_account_db", |b| b.iter(|| 
        place_orders_engine_account(black_box(N_ORDERS_BENCH), base_asset_id, quote_asset_id, &mut account_to_pub_key, &mut bank_controller, &mut spot_controller, &mut rng, &db, true)));

    c.bench_function("place_orders_engine_account_signed", |b| b.iter(|| 
        place_orders_engine_account_signed(black_box(N_ORDERS_BENCH), base_asset_id, quote_asset_id, &mut account_to_pub_key, &mut account_to_signed_msg, &mut bank_controller, &mut spot_controller, &mut rng, &db, false)));

    c.bench_function("place_orders_engine_account_signed_db", |b| b.iter(|| 
        place_orders_engine_account_signed(black_box(N_ORDERS_BENCH), base_asset_id, quote_asset_id, &mut account_to_pub_key, &mut account_to_signed_msg, &mut bank_controller, &mut spot_controller, &mut rng, &db, true)));
    
    #[cfg(feature = "batch")]
    c.bench_function("place_orders_engine_account_batch_signed", |b| b.iter(|| 
        place_orders_engine_account_batch_signed(black_box(N_ORDERS_BENCH), base_asset_id, quote_asset_id, &mut account_to_pub_key, &mut keys_and_signatures, &mut bank_controller, &mut spot_controller, &mut rng, &db, false)));

    #[cfg(feature = "batch")]
    c.bench_function("place_orders_engine_account_batch_signed_db", |b| b.iter(|| 
        place_orders_engine_account_batch_signed(black_box(N_ORDERS_BENCH), base_asset_id, quote_asset_id, &mut account_to_pub_key, &mut keys_and_signatures, &mut bank_controller, &mut spot_controller, &mut rng, &db, true)));

}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
