
extern crate engine;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};
use std::time::SystemTime;

use engine::orderbook::{Orderbook};
use engine::orders;
use engine::domain::OrderSide;

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
fn place_orders(n_orders: u128, rng: &mut ThreadRng) {
    // initialize orderbook
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let mut orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);

    // bench
    let mut i_order: u128 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let qty = round(rng.gen_range(0.0..10.0), 3)+0.001;
        let price = round(rng.gen_range(0.0..10.0), 3)+0.001;

        // order construction & submission
        let order = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            order_type,
            price,
            qty,
            SystemTime::now()
        );
        orderbook.process_order(order);
        
        i_order+=1;
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize orderbook helpers
    let mut rng: ThreadRng = rand::thread_rng();
    c.bench_function("place_orders_engine_only", |b| b.iter(|| place_orders(black_box(100_000), &mut rng)));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
