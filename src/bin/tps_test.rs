
extern crate orderbook;

use std::time::SystemTime;
use orderbook::{Orderbook, OrderSide, orders};
use rand::Rng;

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

fn main() {
    let mut orderbook = Orderbook::new(BrokerAsset::BTC, BrokerAsset::USD);
    let order_asset = parse_asset("BTC").unwrap();
    let price_asset = parse_asset("USD").unwrap();
    let mut rng = rand::thread_rng();
    // let n_orders = 100_000;
    let n_orders = 100;
    let log_freq = 10;
    let mut i_order = 0;

    let now = SystemTime::now();
    while i_order < n_orders {
        // build
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let order = orders::new_limit_order_request(
            order_asset,
            price_asset,
            order_type,
            round(rng.gen_range(0.0..10.0), 3),
            round(rng.gen_range(0.0..10.0), 3),
            SystemTime::now()
        );

        // processing
        if i_order % log_freq == 0 { 
            println!("Order => {:?}", &order);
        }
        let res = orderbook.process_order(order);
        if i_order % log_freq == 0 {
            println!("Processing => {:?}", res);
            if let Some((bid, ask)) = orderbook.current_spread() {
                println!("Spread => bid: {}, ask: {}\n", bid, ask);
            } else {
                println!("Spread => not available\n");
            }
        }
        i_order+=1;
    }
    let time_in_milis = now.elapsed().unwrap().as_millis();
    println!("Processing {} orders took {} milis, giving {} TPS",
                n_orders, time_in_milis, n_orders/time_in_milis*1000); 
}