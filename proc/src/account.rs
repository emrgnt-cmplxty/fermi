//! 
//! 
//! TODO
//! 1.) Add support for market orders
//! 2.) Implement cryptographic verification
//! 3.) Consider ways to avoid passing full mut self in proc_limit_result
//! 
//! 
extern crate engine;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::SystemTime;
use engine::orders;
use engine::domain::{OrderSide};
use engine::orderbook::{Orderbook, OrderProcessingResult, Success, Failed};
use engine::orders::{OrderRequest};


type AccountId = u64;
type OrderId = u64;

// Controller 
// The controller is responsible for performing checks and placing orders on behalf controlled accounts
// The controller updates global account state according to the order output
pub struct MarketController<Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    base_asset: Asset,
    quote_asset: Asset,
    orderbook: Orderbook<Asset>,
    accounts: HashMap<AccountId, Account>,
    order_to_account: HashMap<OrderId, AccountId>,
}

/// Orderbook account
#[derive(Debug)]
pub struct Account {
    pub n_orders: u64,
    pub account_id: AccountId,
    pub base_balance: f64,
    pub base_escrow: f64,
    pub quote_balance: f64,
    pub quote_escrow: f64,
}

impl Account {
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
pub enum AccountError {
    Creation(String),
    Lookup(String),
    OrderProc(String)
}

impl<Asset> MarketController <Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub fn new(base_asset: Asset, quote_asset: Asset) -> Self {
        let mut orderbook: Orderbook<Asset> = Orderbook::new(base_asset, quote_asset);
        MarketController{
            base_asset: base_asset,
            quote_asset: quote_asset,
            orderbook: orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
        }
    }

    // create a controlled account corresponding to account_id
    pub fn create_account(&mut self, account_id: u64, base_balance: f64, quote_balance: f64) -> Result<(), AccountError> {
        if self.accounts.contains_key(&account_id) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.accounts.insert(account_id, Account::new(account_id, base_balance, quote_balance));
            Ok(())
        }
    }

    // get a controlled account corresponding to account_id
    pub fn get_account(&self, account_id: u64) -> Result<&Account, AccountError> {
        let account: &Account = self.accounts.get(&account_id).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        Ok(account)
    }


    // the controller places bid on behalf of an account corresponding to account_id
    pub fn place_limit_order(&mut self, account_id: u64, side: OrderSide, qty: f64, price: f64) -> Result<(), AccountError> {
        // check account has sufficient balances
        {
            let account: &Account = self.accounts.get(&account_id).unwrap();
            if matches!(side, OrderSide::Ask) { 
                assert!(account.base_balance  > qty);
            } else { 
                assert!(account.quote_balance  > qty * price);
            }
        }
        let order: OrderRequest<Asset>= orders::new_limit_order_request(
            self.base_asset,
            self.quote_asset,
            side,
            price,
            qty,
            SystemTime::now()
        );
        let res: Vec<Result<Success, Failed>> = self.orderbook.process_order(order);
        self.proc_limit_result(account_id, side, price, qty, res)
    }

    // loop over and process the output from placing a limit order
    fn proc_limit_result(&mut self, account_id: u64, sub_side: OrderSide, sub_price: f64, sub_qty: f64,  res: OrderProcessingResult) -> Result<(), AccountError> {
        for order in res {
            match order {
                // first order is expected to be an Accepted result
                Ok(Success::Accepted{order_id, ..}) => { 
                    let account: &mut Account = self.accounts.get_mut(&account_id).unwrap();
                    MarketController::<Asset>::proc_order_init(account, sub_side, sub_price, sub_qty);
                    // insert new order to map
                    self.order_to_account.insert(order_id, account_id);
                },
                // subsequent orders are expected to be an PartialFill or Fill results
                Ok(Success::PartiallyFilled{order_id, side, price, qty, ..}) => {
                    let existing_id: &u64 = self.order_to_account.get(&order_id).unwrap();
                    let account: &mut Account = self.accounts.get_mut(&existing_id).unwrap();
                    MarketController::<Asset>::proc_order_fill(account, side, price, qty, 0);
                },
                Ok(Success::Filled{order_id, side, price, qty, ..}) => {
                    let existing_id: &u64 = self.order_to_account.get(&order_id).unwrap();
                    let account: &mut Account = self.accounts.get_mut(&existing_id).unwrap();
                    MarketController::<Asset>::proc_order_fill(account, side, price, qty, -1);
                    // erase existing order
                    self.order_to_account.remove(&order_id).unwrap();
                }
                Ok(Success::Amended { .. }) => { panic!("This needs to be implemented...") }
                Ok(Success::Cancelled { .. }) => { panic!("This needs to be implemented...") }
                Err(_) => { 
                    return Err(AccountError::OrderProc("Order failed to process".to_string()));
                }
            }
        }
        Ok(())
    }

    // process an initialized order by modifying the associated account
    fn proc_order_init(account: &mut Account, side: OrderSide, price: f64, qty: f64) {
        account.n_orders = account.n_orders + 1;
        if matches!(side, OrderSide::Ask) { 
            // E.g. ask 1 BTC @ $20k moves 1 BTC (base) from balance to escrow
            account.base_balance -= qty; 
            account.base_escrow += qty;  
        } else { 
            // E.g. bid 1 BTC @ $20k moves 20k USD (quote) from balance to escrow
            account.quote_balance -= qty * price; 
            account.quote_escrow += qty * price; 
        }
    }
    
    // process a filled order by modifying the associated account
    fn proc_order_fill(account: &mut Account, side: OrderSide, price: f64, qty: f64, order_increment: i64) {
        account.n_orders = (account.n_orders as i64 + order_increment) as u64; 
        if matches!(side, OrderSide::Ask) { 
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            account.quote_balance += qty * price; 
            account.base_escrow -= qty;  
        } else { 
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            account.base_balance += qty; 
            account.quote_escrow -= qty * price; 
        }
    }
}
