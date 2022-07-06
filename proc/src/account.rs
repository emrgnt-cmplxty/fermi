
extern crate engine;

use std::fmt::Debug;
use std::collections::HashMap;
use engine::orders;
use engine::domain::{OrderSide};
use engine::orderbook::{Orderbook, OrderProcessingResult, Success, Failed};
use engine::orders::{OrderRequest};

use std::time::SystemTime;


// for storing local order data
#[derive(Debug)]

/// Controller for account associated to a given market
pub struct MarketController<Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    base_asset: Asset,
    quote_asset: Asset,
    accounts: HashMap<u64, Account>,
    // order id to account id map
    order_to_account: HashMap<u64, u64>,
}

/// Orderbook account
#[derive(Debug)]
pub struct Account
{
    pub n_orders: u64,
    pub account_id: u64,
    pub base_balance: f64,
    pub base_escrow: f64,
    pub quote_balance: f64,
    pub quote_escrow: f64,
}

impl Account
{
    fn new(account_id: u64, base_balance: f64, quote_balance: f64) -> Self {
        Account{
            n_orders: 0, 
            account_id,
            base_balance, 
            base_escrow: 0.0, 
            quote_balance, 
            quote_escrow: 0.0 
        }
    }

}

#[derive(Debug)]
pub enum AccountErrors {
    Creation(String),
    Lookup(String),
    Order(String)
}

impl<Asset> MarketController <Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub fn new(base_asset: Asset, quote_asset: Asset) -> Self {
        MarketController {
            base_asset: base_asset,
            quote_asset: quote_asset,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
        }
    }
    
    fn init_account_order(account: &mut Account, side: OrderSide, price: f64, qty: f64) {
        account.n_orders = account.n_orders + 1;
        if matches!(side, OrderSide::Ask) { 
            // E.g. ask 1 BTC @ $20k requires 1 BTC (base) from balance to escrow
            account.base_balance -= qty; 
            account.base_escrow += qty;  
        } else { 
            // E.g. bid 1 BTC @ $20k nives 20k USD (quote) from balance to escrow
            account.quote_balance -= qty * price; 
            account.quote_escrow += qty * price; 
        };
    }
    
    fn process_account_fill(account: &mut Account, side: OrderSide, price: f64, qty: f64, order_increment: i64) {
        account.n_orders = (account.n_orders as i64 + order_increment) as u64; 
        if matches!(side, OrderSide::Ask) { 
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            account.quote_balance += qty * price; 
            account.base_escrow -= qty;  
        } else { 
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            account.base_balance += qty; 
            account.quote_escrow -= qty * price; 
        };
    }

    // create account inside market controller
    pub fn create_account(&mut self, account_id: u64, base_balance: f64, quote_balance: f64) -> Result<(), AccountErrors> {
        if self.accounts.contains_key(&account_id) {
            Err(AccountErrors::Creation("Account already exists!".to_string()))
        } else {
            self.accounts.insert(account_id, Account::new(account_id, base_balance, quote_balance));
            Ok(())
        }
    }

    // place bid on behalf of account
    pub fn place_limit_order(&mut self, account_id: u64, orderbook: &mut Orderbook<Asset>, side: OrderSide, qty: f64, price: f64) {
        // check account has sufficient balances
        {
            let account: &Account = self.accounts.get(&account_id).unwrap();
            if matches!(side, OrderSide::Ask) { 
                assert!(account.base_balance  > qty);
            } else { 
                assert!(account.quote_balance  > qty * price);
            };
        }
        let order: OrderRequest<Asset>= orders::new_limit_order_request(
            self.base_asset,
            self.quote_asset,
            side,
            price,
            qty,
            SystemTime::now()
        );
        let res: Vec<Result<Success, Failed>> = orderbook.process_order(order);
        self.process_limit_result(account_id, side, price, qty, res).unwrap();
    }

    // process output 
    // TODO - should we consider ways to avoid passing full mut self?
    // Attempts to pass previously loaded account resulted in double borrow err....
    // fn process_order_result(account: &mut Account, accounts: &mut HashMap<u64, Account>, res: OrderProcessingResult) {
    fn process_limit_result(&mut self, account_id: u64, sub_side: OrderSide, sub_price: f64, sub_qty: f64,  res: OrderProcessingResult) -> Result<(), AccountErrors> {
        for order in res {
            match order {
                // first order is expected to be an Accepted result
                Ok(Success::Accepted{order_id, ..}) => { 
                    let account: &mut Account = self.accounts.get_mut(&account_id).unwrap();
                    self.order_to_account.insert(order_id, account_id);
                    // special case handling for initializing order
                    MarketController::<Asset>::init_account_order(account, sub_side, sub_price, sub_qty);
                },
                // subsequent orders are expected to be an PartialFill or Fill results
                Ok(Success::PartiallyFilled{order_id, side, price, qty, ..}) => {
                    let existing_id: &u64 = self.order_to_account.get(&order_id).unwrap();
                    let account: &mut Account = self.accounts.get_mut(&existing_id).unwrap();
                    MarketController::<Asset>::process_account_fill(account, side, price, qty, 0);
                    // over-write existing order
                    self.order_to_account.insert(order_id, account_id);
                },
                Ok(Success::Filled{order_id, side, price, qty, ..}) => {
                    let existing_id: &u64 = self.order_to_account.get(&order_id).unwrap();
                    let account: &mut Account = self.accounts.get_mut(&existing_id).unwrap();
                    MarketController::<Asset>::process_account_fill(account, side, price, qty, -1);
                    // erase existing order
                    self.order_to_account.remove(&order_id).unwrap();
                }
                Ok(Success::Amended { .. }) => { panic!("THIS NEEDS TO BE IMPLEMENTED") }
                Ok(Success::Cancelled { .. }) => { panic!("THIS NEEDS TO BE IMPLEMENTED") }
                Err(_) => { 
                    return Err(AccountErrors::Order("AN UNEXPECTED ERROR WAS ENCOUNTERED".to_string()));
                }
            }
        }
        Ok(())
    }

    pub fn get_account(&self, account_id: u64) -> &Account {
        let account = self.accounts.get(&account_id).unwrap();
        account
    }
}
