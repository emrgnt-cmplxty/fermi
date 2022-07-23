extern crate engine;
extern crate rocksdb;
extern crate types;


use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use engine::{orderbook::Orderbook, orders::new_limit_order_request};
use proc::{account::generate_key_pair, bank::BankController, spot::OrderbookInterface};
use rand::{rngs::ThreadRng, Rng};
use rocksdb::{ColumnFamilyDescriptor, DBWithThreadMode, Options, SingleThreaded, DB};
use std::time::SystemTime;
use types::{
    account::AccountPubKey,
    orderbook::{OrderProcessingResult, OrderSide, Success},
};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;
const TRANSFER_AMOUNT: u64 = 500_000_000;

fn persist_result(db: &DBWithThreadMode<SingleThreaded>, proc_result: &OrderProcessingResult) {
    for result in proc_result {
        match result {
            Ok(Success::Accepted { order_id, .. }) => {
                db.put(order_id.to_string(), "a").unwrap();
                order_id
            }
            Ok(Success::PartiallyFilled { order_id, .. }) => {
                db.put(order_id.to_string(), "pf").unwrap();
                order_id
            }
            Ok(Success::Filled { order_id, .. }) => {
                db.put(order_id.to_string(), "f").unwrap();
                order_id
            }
            _ => &0,
        };
    }
}

fn place_orders_engine(
    base_asset_id: u64,
    quote_asset_id: u64,
    n_orders: u64,
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>,
    persist: bool,
) {
    // initialize orderbook
    let mut orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // bench
    let mut i_order = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        let quantity = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);

        // order construction & submission
        let order = new_limit_order_request(
            base_asset_id,
            quote_asset_id,
            order_type,
            price,
            quantity,
            SystemTime::now(),
        );
        let res = orderbook.process_order(order);
        if persist {
            persist_result(db, &res);
        }
        i_order += 1;
    }
}

fn place_orders_engine_account(
    n_orders: u64,
    base_asset_id: u64,
    quote_asset_id: u64,
    account_to_pub_key: &mut Vec<AccountPubKey>,
    bank_controller: &mut BankController,
    orderbook_controller: &mut OrderbookInterface,
    rng: &mut ThreadRng,
    db: &DBWithThreadMode<SingleThreaded>,
    persist: bool,
) {
    // initialize orderbook
    let orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // clean market controller orderbook to keep from getting clogged
    orderbook_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let quantity = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);
        // generate a random integer between 0 and 100
        let account_pub_key = account_to_pub_key[rng.gen_range(1, 100)];

        let res = orderbook_controller
            .place_limit_order(bank_controller, &account_pub_key, order_type, quantity, price)
            .unwrap();
        if persist {
            persist_result(db, &res);
        }
        i_order += 1;
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    // generate creator details
    let (creator_key, _private_key) = generate_key_pair();

    // initialize market controller
    let mut account_to_pub_key: Vec<AccountPubKey> = Vec::new();
    let mut bank_controller = BankController::new();
    let base_asset_id = bank_controller.create_asset(&creator_key).unwrap();
    let quote_asset_id = bank_controller.create_asset(&creator_key).unwrap();

    let mut orderbook_controller = OrderbookInterface::new(base_asset_id, quote_asset_id);
    orderbook_controller.create_account(&creator_key).unwrap();
    // other helpers
    let mut i_account = 0;
    let path = "./db.rocks";
    let mut cf_opts = Options::default();
    cf_opts.set_max_write_buffer_number(16);
    let cf = ColumnFamilyDescriptor::new("cf1", cf_opts);
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);
    let db = DB::open_cf_descriptors(&db_opts, path, vec![cf]).unwrap();

    // generate N_ACCOUNTS accounts to transact w/ orderbook
    while i_account < N_ACCOUNTS - 1 {
        let (account_pub_key, _private_key) = generate_key_pair();

        bank_controller
            .transfer(&creator_key, &account_pub_key, base_asset_id, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(&creator_key, &account_pub_key, quote_asset_id, TRANSFER_AMOUNT)
            .unwrap();

        // create initial asset
        account_to_pub_key.push(account_pub_key);
        i_account += 1;
    }

    let mut group = c.benchmark_group("orderbook");
    group.throughput(Throughput::Elements((N_ORDERS_BENCH) as u64));

    // no write-out to db
    group.bench_function("place_orders_engine", |b| {
        b.iter(|| {
            place_orders_engine(
                base_asset_id,
                quote_asset_id,
                black_box(N_ORDERS_BENCH),
                &mut rng,
                &db,
                false,
            )
        })
    });

    group.bench_function("place_orders_engine_account", |b| {
        b.iter(|| {
            place_orders_engine_account(
                black_box(N_ORDERS_BENCH),
                base_asset_id,
                quote_asset_id,
                &mut account_to_pub_key,
                &mut bank_controller,
                &mut orderbook_controller,
                &mut rng,
                &db,
                false,
            )
        })
    });

    // w/ write-out to db
    group.bench_function("place_orders_engine_db", |b| {
        b.iter(|| {
            place_orders_engine(
                base_asset_id,
                quote_asset_id,
                black_box(N_ORDERS_BENCH),
                &mut rng,
                &db,
                true,
            )
        })
    });

    group.bench_function("place_orders_engine_account_db", |b| {
        b.iter(|| {
            place_orders_engine_account(
                black_box(N_ORDERS_BENCH),
                base_asset_id,
                quote_asset_id,
                &mut account_to_pub_key,
                &mut bank_controller,
                &mut orderbook_controller,
                &mut rng,
                &db,
                true,
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
