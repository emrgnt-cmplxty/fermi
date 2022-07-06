
extern crate engine;

use rand::{Rng};
use rand::rngs::{ThreadRng};
use std::time::SystemTime;
use engine::orderbook::{Orderbook};
use engine::orders;
use engine::domain::OrderSide;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

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
fn place_orders(n_orders: u128, base_asset:BrokerAsset, quote_asset:BrokerAsset, orderbook: &mut Orderbook<BrokerAsset>, rng: &mut ThreadRng) {

    let mut i_order: u128 = 0;
    let now: SystemTime = SystemTime::now();
    while i_order < n_orders {
        // inputs
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let qty = round(rng.gen_range(0.0..10.0), 3);
        let price = round(rng.gen_range(0.0..10.0), 3);

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
        i_order+=1;
    }
    // let time_in_nanos: u128 = now.elapsed().unwrap().as_nanos();
    // println!("Processing {} orders took {} nanos, giving {} TPS",
    //             n_orders, time_in_nanos, (n_orders as f64)/(time_in_nanos as f64) * 1e9); 

}

pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize orderbook helpers
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let mut orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);
    let mut rng: ThreadRng = rand::thread_rng();

    c.bench_function("place_orders", |b| b.iter(|| place_orders(black_box(10_000), base_asset, quote_asset, &mut orderbook, &mut rng)));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
