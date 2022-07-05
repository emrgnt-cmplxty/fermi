
extern crate engine;
extern crate proc;

use rand::Rng;
use std::time::SystemTime;
use engine::orderbook::{Orderbook};
use engine::orders;
use engine::domain::OrderSide;    
use proc::account::{Account, MAX_N};
use assert_approx_eq::assert_approx_eq;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BrokerAsset {
    BTC,
    USD,
}

fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i64.pow(decimals) as f64;
    (x * y).round() / y
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmatched_orders() {
        let base_asset = BrokerAsset::BTC;
        let quote_asset = BrokerAsset::USD;
        let base_balance = 1_000.0;
        let quote_balance = 1_000.0;

        let mut rng = rand::thread_rng();

        let mut orderbook = Orderbook::new(base_asset, quote_asset);
        let mut account = Account::new(base_balance, quote_balance, base_asset, quote_asset);

        let mut i_order = 0;
        let mut sum_orders: f64 = 0.0;

        // loop over 10 bids and check closure
        while i_order < 10 {
            let amount: f64 = round(rng.gen_range(0.0..10.0), 3);
            let price: f64 = round(rng.gen_range(0.0..10.0), 3);
            sum_orders += amount;
            account.place_bid_order(&mut orderbook, amount, price);
            assert_approx_eq!(account.get_quote_balance(), quote_balance - sum_orders);
            i_order += 1;
        }

        let mut i_order = 0;
        let mut sum_orders: f64 = 0.0;
        // loop over 10 asks and check closure
        while i_order < 10 {
            let amount: f64 = round(rng.gen_range(0.0..10.0), 3);
            let price: f64 = round(rng.gen_range(0.0..10.0), 3);
            sum_orders += amount;
            account.place_ask_order(&mut orderbook, amount, price);
            assert_approx_eq!(account.get_base_balance(), base_balance - sum_orders);
            i_order += 1;
        }
        
    }
}