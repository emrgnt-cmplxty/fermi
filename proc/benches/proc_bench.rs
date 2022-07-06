
extern crate engine;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};

use engine::domain::OrderSide;
use proc::account::{Account, AccountController};

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

#[inline]
fn place_orders(n_orders: u64, n_accounts: u64, rng: &mut ThreadRng) {
    // initialize market controller
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    // let account_id: u64 = 0;
    let base_balance: f64 = 1_000_000_000.0;
    let quote_balance: f64 = 1_000_000_000.0;
    let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

    let mut i_account: u64 = 0;
    let mut i_order: u64 = 0;

    while i_account < n_accounts{
        market_controller.create_account(i_account, base_balance, quote_balance).unwrap();
        i_account += 1;
    }

    // bench
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let qty = round(rng.gen_range(0.0..10.0), 3)+0.001;
        let price = round(rng.gen_range(0.0..10.0), 3)+0.001;
        let account_id = rng.gen_range(0..100);

        market_controller.place_limit_order(account_id,  order_type, qty, price).unwrap();

        i_order+=1;
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize market controller helpers
    let mut rng: ThreadRng = rand::thread_rng();
    c.bench_function("place_orders_engine_plus_account", |b| b.iter(|| place_orders(black_box(100_000), black_box(100),&mut rng)));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
