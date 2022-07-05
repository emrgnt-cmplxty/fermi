
extern crate engine;

use std::fmt::Debug;
// use engine::orderbook::{OrderSide, orders};
use engine::orders;
use engine::domain::OrderSide;

use std::time::SystemTime;
pub const MAX_N: usize = 1000;

pub struct Account<Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    base_asset: Asset,
    quote_asset: Asset,
    orders: [u64; MAX_N],
    base_balance: f64,
    order_base_escrow: f64,
    quote_balance: f64,
    order_quote_escrow: f64,
}

impl<Asset> Account <Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub fn new(base_balance: f64, quote_balance: f64, 
                base_asset: Asset, quote_asset: Asset) -> Self {
        Account{
            base_asset,
            quote_asset,
            orders: [0; MAX_N], 
            base_balance: base_balance, 
            order_base_escrow: 0.0, 
            quote_balance: quote_balance, 
            order_quote_escrow: 0.0 
        }
    }

    pub fn place_bid_order(&mut self, order_type: OrderSide, amount: f64, price: f64) {
        assert!(self.quote_balance > amount);
        let order = orders::new_limit_order_request(
            self.base_asset,
            self.quote_asset,
            order_type,
            amount,
            price,
            SystemTime::now()
        );
        self.quote_balance = self.quote_balance - amount;
    }

    pub fn get_quote_balance(&self) -> f64 {
        self.quote_balance
    }

}
