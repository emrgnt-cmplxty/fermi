
extern crate engine;
extern crate user;

use rand::Rng;
use std::time::SystemTime;
use engine::orderbook::{Orderbook};
use engine::orders;
use engine::domain::OrderSide;
use user::account::{Account, MAX_N};
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
        let mut sum_bids: f64 = 0.0;
        let mut sum_asks: f64 = 0.0;

        // loop over 10 bids & asks and check closure
        while i_order < 10 {
            let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
            let amount: f64 = round(rng.gen_range(0.0..10.0), 3);
            let price: f64 = round(rng.gen_range(0.0..10.0), 3);
            sum_bids += amount;
            account.place_bid_order(OrderSide::Bid, amount, price);
            println!("sum_bids={}", sum_bids);
            assert_approx_eq!(account.get_quote_balance(), quote_balance - sum_bids);
            i_order += 1;
        }
    }
}