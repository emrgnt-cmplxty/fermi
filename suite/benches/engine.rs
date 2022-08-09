use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use gdex_controller::{bank::BankController, spot::OrderbookInterface};
use gdex_engine::{order_book::Orderbook, orders::new_limit_order_request};
use gdex_types::transaction::OrderRequest;
use gdex_types::{
    account::{account_test_functions::generate_keypair_vec, AccountPubKey},
    crypto::KeypairTraits,
    order_book::{OrderProcessingResult, OrderSide, Success},
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rocksdb::{ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded, Options, DB};
use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;
const TRANSFER_AMOUNT: u64 = 500_000_000;

fn persist_result(db: &DBWithThreadMode<MultiThreaded>, proc_result: &OrderProcessingResult) {
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
    rng: &mut StdRng,
    db: &DBWithThreadMode<MultiThreaded>,
    persist: bool,
) {
    // initialize orderbook
    let mut orderbook: Orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type: OrderSide = if i_order % 2 == 0 {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        let quantity: u64 = rng.gen_range(1, 100);
        let price: u64 = rng.gen_range(1, 100);

        // order construction & submission
        let order: OrderRequest = new_limit_order_request(
            base_asset_id,
            quote_asset_id,
            order_type,
            price,
            quantity,
            SystemTime::now(),
        );
        let res: OrderProcessingResult = orderbook.process_order(order);
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
    primary: &AccountPubKey,
    orderbook_controller: &mut OrderbookInterface,
    rng: &mut StdRng,
    db: &DBWithThreadMode<MultiThreaded>,
    persist: bool,
) {
    // initialize orderbook
    let orderbook: Orderbook = Orderbook::new(base_asset_id, quote_asset_id);

    // clean market controller orderbook to keep from getting clogged
    orderbook_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type: OrderSide = if i_order % 2 == 0 {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let quantity = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);

        let res = orderbook_controller
            .place_limit_order(primary, order_type, quantity, price)
            .unwrap();
        if persist {
            persist_result(db, &res);
        }
        i_order += 1;
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // generate creator details
    let primary = generate_keypair_vec([0; 32]).pop().unwrap();

    // initialize market controller
    let mut account_to_pub_key: Vec<AccountPubKey> = Vec::new();
    let mut bank_controller: BankController = BankController::new();
    bank_controller.create_asset(&primary.public()).unwrap();
    let base_asset_id = 0;
    bank_controller.create_asset(&primary.public()).unwrap();
    let quote_asset_id = 1;

    let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

    let mut orderbook_interface =
        OrderbookInterface::new(base_asset_id, quote_asset_id, Arc::clone(&bank_controller_ref));

    orderbook_interface.create_account(&primary.public()).unwrap();
    // other helpers
    let mut i_account: u64 = 0;
    let path: &str = "./db.rocks";
    let mut cf_opts: Options = Options::default();
    cf_opts.set_max_write_buffer_number(16);
    let cf: ColumnFamilyDescriptor = ColumnFamilyDescriptor::new("cf1", cf_opts);
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);
    let db = DB::open_cf_descriptors(&db_opts, path, vec![cf]).unwrap();

    let mut rng = StdRng::from_seed([2; 32]);
    // generate N_ACCOUNTS accounts to transact w/ orderbook
    while i_account < N_ACCOUNTS - 1 {
        let receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        bank_controller_ref
            .lock()
            .unwrap()
            .transfer(&primary.public(), &receiver.public(), base_asset_id, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller_ref
            .lock()
            .unwrap()
            .transfer(&primary.public(), &receiver.public(), quote_asset_id, TRANSFER_AMOUNT)
            .unwrap();

        // create initial asset
        account_to_pub_key.push(receiver.public().clone());
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
                primary.public(),
                &mut orderbook_interface,
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
                primary.public(),
                &mut orderbook_interface,
                &mut rng,
                &db,
                true,
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);