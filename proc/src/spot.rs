//! 
//! TODO
//! 1.) Add support for market orders
//! 2.) Properly implement cryptographic verification
//! 3.) Consider ways to avoid passing full mut self in proc_limit_result
//! 4.) Limit overwrite_orderbook to bench-only mode
//! 5.) replace dummy_message encryption scheme w/ smarter & more realistic solution
//! 6.) What to do with TestDiemCrypto?
//! 7.) Protect overwrite_orderbook access to bench mode
//! 
extern crate engine;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use engine::orders;
use engine::domain::{OrderSide};
use engine::orderbook::{Orderbook, OrderProcessingResult, Success, Failed};
use engine::orders::{OrderRequest};
use diem_crypto::{traits::{Signature}};
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use super::account::{AccountError, AccountPubKey, AccountSignature, OrderAccount};
use super::bank::{BankController};

type OrderId = u64;

#[derive(Debug, BCSCryptoHash, CryptoHasher, Serialize, Deserialize)]
pub struct TestDiemCrypto(pub String);
// dummy msg used for test-encoding
pub const DUMMY_MESSAGE: &str = "dummy_val";

// The spot controller is responsible for accessing & modifying user orders 
pub struct SpotController <'a, Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    base_asset_id: Asset,
    quote_asset_id: Asset,
    orderbook: Orderbook<Asset>,
    accounts: HashMap<AccountPubKey, OrderAccount>,
    order_to_account: HashMap<OrderId, AccountPubKey>,
    bank_controller: &'a mut BankController,
}


impl<'a, Asset> SpotController <'a, Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub fn new(base_asset_id: Asset, quote_asset_id: Asset, bank_controller: &'a mut BankController) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook: Orderbook<Asset> = Orderbook::new(base_asset_id, quote_asset_id);
        SpotController{
            base_asset_id,
            quote_asset_id,
            orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
            bank_controller
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey, base_balance: u64, quote_balance: u64) -> Result<(), AccountError> {
        if self.accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.accounts.insert(*account_pub_key, OrderAccount::new(*account_pub_key, base_balance, quote_balance));
            Ok(())
        }
    }

    pub fn get_account(&self, account_pub_key: &AccountPubKey) -> Result<&OrderAccount, AccountError> {
        let account: &OrderAccount = self.accounts.get(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        Ok(account)
    }

    pub fn place_limit_order(&mut self, account_pub_key: &AccountPubKey, side: OrderSide, qty: u64, price: u64) -> Result<OrderProcessingResult, AccountError> {
        // check balances before placing order
        let account: &OrderAccount = self.accounts.get(account_pub_key).unwrap();
        if matches!(side, OrderSide::Ask) { 
            assert!(account.base_balance  > qty);
        } else { 
            assert!(account.quote_balance  > qty * price);
        }
        // create and process limit order
        let order: OrderRequest<Asset>= orders::new_limit_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            side,
            price,
            qty,
            SystemTime::now()
        );
        let res: Vec<Result<Success, Failed>> = self.orderbook.process_order(order);
        self.proc_limit_result(account_pub_key, side, price, qty, res)
    }

    // signed workflow
    // TODO - flesh out more
    pub fn place_signed_limit_order(&mut self, account_pub_key: &AccountPubKey, side: OrderSide, qty: u64, price: u64, signed_message: &AccountSignature) -> Result<OrderProcessingResult, AccountError> {
        signed_message.verify(&TestDiemCrypto(DUMMY_MESSAGE.to_string()), &account_pub_key).unwrap();
        self.place_limit_order(account_pub_key, side, qty, price)
    }

    // loop over and process the output from placing a limit order
    fn proc_limit_result(&mut self, account_pub_key: &AccountPubKey, sub_side: OrderSide, sub_price: u64, sub_qty: u64,  res: OrderProcessingResult) -> Result<OrderProcessingResult, AccountError> {
        for order in &res {
            match order {
                // first order is expected to be an Accepted result
                Ok(Success::Accepted{order_id, ..}) => { 
                    let account: &mut OrderAccount = self.accounts.get_mut(&account_pub_key).unwrap();
                    SpotController::<Asset>::proc_order_init(account, sub_side, sub_price, sub_qty);
                    // insert new order to map
                    self.order_to_account.insert(*order_id, *account_pub_key);
                },
                // subsequent orders are expected to be an PartialFill or Fill results
                Ok(Success::PartiallyFilled{order_id, side, price, qty, ..}) => {
                    let existing_pub_key: &AccountPubKey = self.order_to_account.get(&order_id).unwrap();
                    let account: &mut OrderAccount = self.accounts.get_mut(&existing_pub_key).unwrap();
                    SpotController::<Asset>::proc_order_fill(account, *side, *price, *qty, 0);
                },
                Ok(Success::Filled{order_id, side, price, qty, ..}) => {
                    let existing_pub_key: &AccountPubKey = self.order_to_account.get(&order_id).unwrap();
                    let account: &mut OrderAccount = self.accounts.get_mut(&existing_pub_key).unwrap();
                    SpotController::<Asset>::proc_order_fill(account, *side, *price, *qty, -1);
                    // erase existing order
                    self.order_to_account.remove(&order_id).unwrap();
                }
                Ok(Success::Amended { .. }) => { panic!("This needs to be implemented...") }
                Ok(Success::Cancelled { .. }) => { panic!("This needs to be implemented...") }
                Err(failure) => { 
                    return Err(AccountError::OrderProc(format!("Order failed to process with {:?}", failure)));
                }
            }
        }
        Ok(res)
    }

    // process an initialized order by modifying the associated account
    fn proc_order_init(account: &mut OrderAccount, side: OrderSide, price: u64, qty: u64) {
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
    fn proc_order_fill(account: &mut OrderAccount, side: OrderSide, price: u64, qty: u64, order_increment: i64) {
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
    // TODO - can we guard this to only be accessible in "bench" mode?
    // e.g. like #[cfg(bench)], except this only works locally
    pub fn overwrite_orderbook(&mut self, new_orderbook: Orderbook<Asset>) {
        self.orderbook = new_orderbook;
    }

}
