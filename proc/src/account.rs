
extern crate engine;

use std::fmt::Debug;
// use engine::orderbook::{OrderSide, orders};
use engine::orders;
use engine::domain::OrderSide;
use engine::orderbook::{Orderbook, OrderProcessingResult, Success};

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
    base_escrow: f64,
    quote_balance: f64,
    quote_escrow: f64,
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
            base_escrow: 0.0, 
            quote_balance: quote_balance, 
            quote_escrow: 0.0 
        }
    }

    pub fn place_bid_order(&mut self, orderbook: &mut Orderbook<Asset>, amount: f64, price: f64) {
        assert!(self.quote_balance > amount);
        let order = orders::new_limit_order_request(
            self.base_asset,
            self.quote_asset,
            OrderSide::Bid,
            amount,
            price,
            SystemTime::now()
        );
        println!("Order => {:?}", &order);
        let res = orderbook.process_order(order);
        self.process_bid_result(res);
        // println!("res => {:?}", &res);
        self.quote_balance = self.quote_balance - amount;
    }

    pub fn process_bid_result(&mut self, res: OrderProcessingResult) {
        println!("res => {:?}", &res);
        println!("res.len => {:?}", res.len());
        for order in res {
            match order {
                Ok(Success::Accepted{id, order_type, ts}) => println!("accepted... {}", id),
                Ok(Success::PartiallyFilled{order_id, side, order_type, price, qty, ts}) => println!("partial filled {} with quantity {}", order_id, qty),
                Ok(Success::Filled{order_id, qty, ..}) => println!("filled {} with quantity {}", order_id, qty),
                _ => println!("other..."),
            }
        }
    }

    pub fn place_ask_order(&mut self, orderbook: &mut Orderbook<Asset>, amount: f64, price: f64) {
        assert!(self.base_balance > amount);
        let order = orders::new_limit_order_request(
            self.base_asset,
            self.quote_asset,
            OrderSide::Ask,
            amount,
            price,
            SystemTime::now()
        );
        println!("Order => {:?}", &order);
        let res = orderbook.process_order(order);
        println!("res => {:?}", &res);
        self.process_ask_result(res);
        self.base_balance = self.base_balance - amount;
    }

    pub fn process_ask_result(&mut self, res: OrderProcessingResult) {
        println!("res => {:?}", &res);
        println!("res.len => {:?}", res.len());
        for order in res {
            match order {
                Ok(Success::Accepted{id, ..}) => println!("accepted id={}", id),
                Ok(Success::PartiallyFilled{order_id, qty, ..}) => println!("partially filled id={} with quantity {}", order_id, qty),
                Ok(Success::Filled{order_id, qty, ..}) => println!("filled id={} with quantity {}", order_id, qty),
                _ => println!("other..."),
            }
        }
    }


    pub fn get_base_balance(&self) -> f64 {
        self.base_balance
    }

    pub fn get_quote_balance(&self) -> f64 {
        self.quote_balance
    }
}

