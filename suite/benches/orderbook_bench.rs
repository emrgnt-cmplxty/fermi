
extern crate engine;
extern crate rocksdb;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};
use rocksdb::{ColumnFamilyDescriptor, DB, DBWithThreadMode, Options, SingleThreaded};
use std::time::SystemTime;

use diem_crypto::{
    traits::{Signature, SigningKey, Uniform},
    
};
use engine::orders;
use engine::orderbook::{Orderbook, OrderProcessingResult, Success};
use engine::domain::OrderSide;
use proc::account::{AccountPubKey, AccountPrivKey, AccountSignature, AccountController, TestDiemCrypto, DUMMY_MESSAGE};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BrokerAsset {
    USD,
    EUR,
    BTC,
    ETH,
}

fn parse_asset(asset: &str) -> Option<BrokerAsset> {
    match asset {
        "USD" => Some(BrokerAsset::USD),
        "EUR" => Some(BrokerAsset::EUR),
        "BTC" => Some(BrokerAsset::BTC),
        "ETH" => Some(BrokerAsset::ETH),
        _ => None,
    }
}

fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i64.pow(decimals) as f64;
    (x * y).round() / y
}

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
    n_orders: u64, 
    rng: &mut ThreadRng, 
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let mut orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let qty = round(rng.gen_range(0.0, 10.0), 3)+0.001;
        let price = round(rng.gen_range(0.0, 10.0), 3)+0.001;

        // order construction & submission
        let order = orders::new_limit_order_request(
            base_asset,
            quote_asset,
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
    account_to_pub_key: &mut Vec<AccountPubKey>, 
    market_controller: &mut AccountController<BrokerAsset>, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);

    // clean market controller orderbook to keep from getting clogged
    market_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = round(rng.gen_range(0.0, 10.0), 3) + 0.001;
        let price = round(rng.gen_range(0.0, 10.0), 3) + 0.001;
        // generate a random integer between 0 and 100
        let account_pub_key: AccountPubKey = account_to_pub_key[rng.gen_range(0, 100)];

        let res = market_controller.place_limit_order(&account_pub_key,  order_type, qty, price).unwrap();
        if persist {
            persist_result(db, &res);
        }
        i_order+=1;
    }
}
    
fn place_orders_engine_account_signed(
    n_orders: u64, 
    account_to_pub_key: &mut Vec<AccountPubKey>, 
    account_to_signed_msg: &mut Vec<AccountSignature>, 
    market_controller: &mut AccountController<BrokerAsset>, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    // initialize orderbook
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);

    // clean market controller orderbook to keep from getting clogged
    market_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = round(rng.gen_range(0.0, 10.0), 3) + 0.001;
        let price = round(rng.gen_range(0.0, 10.0), 3) + 0.001;
        // generate a random integer between 0 and 100
        let i_account = rng.gen_range(0, 100);
        let account_pub_key: AccountPubKey = account_to_pub_key[i_account];
        let account_sig_msg: &AccountSignature = &account_to_signed_msg[i_account];

        let res = market_controller.place_signed_limit_order(&account_pub_key,  order_type, qty, price, account_sig_msg).unwrap();

        i_order+=1;

        if persist {
            persist_result(db, &res);
        }
    }
}

#[cfg(feature = "batch")]
fn place_orders_engine_account_batch_signed(
    n_orders: u64, 
    account_to_pub_key: &mut Vec<AccountPubKey>, 
    keys_and_signatures: &mut Vec<(AccountPubKey, AccountSignature)>, 
    market_controller: &mut AccountController<BrokerAsset>, 
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>, 
    persist: bool) 
{
    Signature::batch_verify(&TestDiemCrypto(DUMMY_MESSAGE.to_string()), keys_and_signatures.to_vec()).unwrap();
    place_orders_engine_account(n_orders, account_to_pub_key, market_controller, rng, db, persist);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize market controller
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let base_balance: f64 = 1_000_000_000.0;
    let quote_balance: f64 = 1_000_000_000.0;
    let mut account_to_pub_key: Vec<AccountPubKey> = Vec::new();
    let mut account_to_signed_msg: Vec<AccountSignature> = Vec::new();
    let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

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

    
    // generate N_ACCOUNTS accounts to transact w/ orderbook
    while i_account < N_ACCOUNTS{
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();
        account_to_pub_key.push(account_pub_key);
        let sig: AccountSignature  = private_key.sign(&TestDiemCrypto(DUMMY_MESSAGE.to_string()));
        account_to_signed_msg.push(sig.clone());
        market_controller.create_account(&account_pub_key, base_balance, quote_balance).unwrap();
        i_account += 1;
        keys_and_signatures.push((account_pub_key, sig));
    }
    
    c.bench_function("place_orders_engine", |b| b.iter(|| 
        place_orders_engine(black_box(N_ORDERS_BENCH), &mut rng, &db, false)));

    c.bench_function("place_orders_engine_db", |b| b.iter(|| 
        place_orders_engine(black_box(N_ORDERS_BENCH), &mut rng, &db, true)));
    
    c.bench_function("place_orders_engine_account", |b| b.iter(|| 
        place_orders_engine_account(black_box(N_ORDERS_BENCH), &mut account_to_pub_key, &mut market_controller, &mut rng, &db, false)));
    
    c.bench_function("place_orders_engine_account_db", |b| b.iter(|| 
        place_orders_engine_account(black_box(N_ORDERS_BENCH), &mut account_to_pub_key, &mut market_controller, &mut rng, &db, true)));

    c.bench_function("place_orders_engine_account_signed", |b| b.iter(|| 
        place_orders_engine_account_signed(black_box(N_ORDERS_BENCH), &mut account_to_pub_key, &mut account_to_signed_msg, &mut market_controller, &mut rng, &db, false)));

    c.bench_function("place_orders_engine_account_signed_db", |b| b.iter(|| 
        place_orders_engine_account_signed(black_box(N_ORDERS_BENCH), &mut account_to_pub_key, &mut account_to_signed_msg, &mut market_controller, &mut rng, &db, true)));
    
    #[cfg(feature = "batch")]
    c.bench_function("place_orders_engine_account_batch_signed", |b| b.iter(|| 
        place_orders_engine_account_batch_signed(black_box(N_ORDERS_BENCH), &mut account_to_pub_key, &mut keys_and_signatures, &mut market_controller, &mut rng, &db, false)));

    #[cfg(feature = "batch")]
    c.bench_function("place_orders_engine_account_batch_signed_db", |b| b.iter(|| 
        place_orders_engine_account_batch_signed(black_box(N_ORDERS_BENCH), &mut account_to_pub_key, &mut keys_and_signatures, &mut market_controller, &mut rng, &db, true)));

}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
