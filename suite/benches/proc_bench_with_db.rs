
extern crate engine;
extern crate rocksdb;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};

use engine::domain::OrderSide;
use proc::account::{AccountController};
use rocksdb::{ColumnFamilyDescriptor, DB, DBWithThreadMode, Options, SingleThreaded};
use engine::orderbook::{OrderProcessingResult, Success};

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

pub fn persist_result(db: &DBWithThreadMode<SingleThreaded>, proc_result: &OrderProcessingResult) -> () {
    for result in proc_result {
        let id = match result {
            Ok(Success::Accepted { order_id, .. }) => {
                db.put(order_id.to_string(), "a");
                order_id
            },
            Ok(Success::PartiallyFilled { order_id, qty, .. }) => {
                db.put(order_id.to_string(), "pf");
                order_id
            },
            Ok(Success::Filled { order_id, qty, .. }) => {
                db.put(order_id.to_string(), "f");
                order_id
            },
            _ => &0
        };
    }
}

#[inline]
fn place_orders(n_orders: u64, n_accounts: u64, rng: &mut ThreadRng, db: &DBWithThreadMode<SingleThreaded>) {
    // initialize market controller
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    // specify large base balances to avoid failures
    let base_balance: f64 = 1_000_000_000.0;
    let quote_balance: f64 = 1_000_000_000.0;
    let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

    // generaet 100 accounts to transact w/ orderbook
    let mut i_account: u64 = 0;
    while i_account < n_accounts{
        market_controller.create_account(i_account, base_balance, quote_balance).unwrap();
        i_account += 1;
    }

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = round(rng.gen_range(0.0..10.0), 3) + 0.001;
        let price = round(rng.gen_range(0.0..10.0), 3) + 0.001;
        // generate a random integer between 0 and 100
        let account_id = rng.gen_range(0..100);

        let res = market_controller.place_limit_order(account_id,  order_type, qty, price).unwrap();
        persist_result(db, &res);

        i_order+=1;
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize market controller helpers
    let mut rng: ThreadRng = rand::thread_rng();
    let path = "./db.rocks";
    let mut cf_opts = Options::default();
    cf_opts.set_max_write_buffer_number(16);
    let cf = ColumnFamilyDescriptor::new("cf1", cf_opts);
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);
    let db = DB::open_cf_descriptors(&db_opts, path, vec![cf]).unwrap();
    c.bench_function("place_orders_engine_account_db", |b| b.iter(|| place_orders(black_box(100_000), black_box(100),&mut rng, &db)));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
