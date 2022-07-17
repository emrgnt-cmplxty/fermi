//! 
//! TODO
//! 1.) Add support for market orders
//! 2.) Properly implement cryptographic verification
//! 3.) Consider ways to avoid passing full mut self in proc_limit_result
//! 4.) Limit overwrite_orderbook to bench-only mode
//! 5.) replace dummy_message encryption scheme w/ smarter & more realistic solution
//! 6.) What to do with TestDiemCrypto?
//! 7.) Protect overwrite_orderbook access to bench mode
//! 8.) Re-implement escrow & order counting if helpful
//! 9.) Check assets for orderbook exist on creation
//! 10.) Check passed asset ids on order placement
//! 
extern crate core;
extern crate engine;
extern crate types;

use std::{
    collections::HashMap,
    time::SystemTime
};

use super::{
    account::{OrderAccount},
    bank::{BankController}
};
use core::{
    transaction::{
        TxnRequest, 
        TxnVariant,
    }
};
use diem_crypto::{traits::{Signature}};
use engine::{
    orderbook::{Orderbook},
    orders::{OrderRequest, new_limit_order_request}
};
use types::{
    account::{AccountError, AccountPubKey, AccountSignature},
    asset::{AssetId},
    orderbook::{Failed, OrderSide, OrderProcessingResult, Success},
    spot::{OrderId, TestDiemCrypto},
};

// dummy msg used for test-encoding
pub const DUMMY_MESSAGE: &str = "dummy_val";

// The spot controller is responsible for accessing & modifying user orders 
pub struct SpotController
{
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    orderbook: Orderbook,
    accounts: HashMap<AccountPubKey, OrderAccount>,
    order_to_account: HashMap<OrderId, AccountPubKey>,
}


impl SpotController
{
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook: Orderbook = Orderbook::new(base_asset_id, quote_asset_id);
        SpotController{
            base_asset_id,
            quote_asset_id,
            orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        if self.accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.accounts.insert(*account_pub_key, OrderAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn get_account(&self, account_pub_key: &AccountPubKey) -> Result<&OrderAccount, AccountError> {
        let account: &OrderAccount = self.accounts.get(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        Ok(account)
    }

    pub fn place_limit_order(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, side: OrderSide, qty: u64, price: u64) -> Result<OrderProcessingResult, AccountError> {
        // check balances before placing order
        if matches!(side, OrderSide::Ask) { 
            // if ask, selling qty of base asset
            assert!(bank_controller.get_balance(account_pub_key, self.base_asset_id)  > qty);
        } else { 
            // if bid, buying base asset with qty*price of quote asset
            assert!(bank_controller.get_balance(account_pub_key, self.quote_asset_id)   > qty * price);
        }
        // create and process limit order
        let order: OrderRequest = new_limit_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            side,
            price,
            qty,
            SystemTime::now()
        );
        let res: Vec<Result<Success, Failed>> = self.orderbook.process_order(order);
        self.proc_limit_result(bank_controller, account_pub_key, side, price, qty, res)
    }

    pub fn parse_limit_order_txn(&mut self, bank_controller: &mut BankController, signed_txn: &TxnRequest<TxnVariant>) -> Result<OrderProcessingResult, AccountError> {
        // verify transaction is an order
        if let TxnVariant::OrderTransaction(order) = &signed_txn.txn {
            // verify and place a limit order
            if let OrderRequest::NewLimitOrder{side, price, qty, ..} = order.request {
                return self.place_limit_order(bank_controller, &signed_txn.sender_address, side, qty, price)
            } else {
                return Err(AccountError::OrderProc("Only limit orders supported".to_string()))
            }
        } else {
            return Err(AccountError::OrderProc("Only order transactions are supported".to_string()))
        };
    }

    // loop over and process the output from placing a limit order
    fn proc_limit_result(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, sub_side: OrderSide, sub_price: u64, sub_qty: u64,  res: OrderProcessingResult) -> Result<OrderProcessingResult, AccountError> {
        for order in &res {
            match order {
                // first order is expected to be an Accepted result
                Ok(Success::Accepted{order_id, ..}) => { 
                    self.proc_order_init(bank_controller, &account_pub_key, sub_side, sub_price, sub_qty);
                    // insert new order to map
                    self.order_to_account.insert(*order_id, *account_pub_key);
                },
                // subsequent orders are expected to be an PartialFill or Fill results
                Ok(Success::PartiallyFilled{order_id, side, price, qty, ..}) => {
                    let existing_pub_key: AccountPubKey = *self.order_to_account.get(&order_id).unwrap();
                    self.proc_order_fill(bank_controller, &existing_pub_key, *side, *price, *qty);
                },
                Ok(Success::Filled{order_id, side, price, qty, ..}) => {
                    let existing_pub_key: AccountPubKey = *self.order_to_account.get(&order_id).unwrap();
                    self.proc_order_fill(bank_controller,   &existing_pub_key, *side, *price, *qty);
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
    fn proc_order_init(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, side: OrderSide, price: u64, qty: u64) {
        if matches!(side, OrderSide::Ask) { 
            // E.g. ask 1 BTC @ $20k moves 1 BTC (base) from balance to escrow
            bank_controller.update_balance(account_pub_key, self.base_asset_id, -(qty as i64));
        } else { 
            // E.g. bid 1 BTC @ $20k moves 20k USD (quote) from balance to escrow
            bank_controller.update_balance(account_pub_key, self.quote_asset_id, -((qty * price) as i64));
        }
    }
    
    // process a filled order by modifying the associated account
    fn proc_order_fill(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, side: OrderSide, price: u64, qty: u64) {
        if matches!(side, OrderSide::Ask) { 
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            bank_controller.update_balance(account_pub_key, self.quote_asset_id, (qty*price) as i64);
        } else { 
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            bank_controller.update_balance(account_pub_key, self.base_asset_id, qty as i64);
        }
    }


    // signed workflow
    // TODO - flesh out more
    pub fn place_signed_limit_order(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, side: OrderSide, qty: u64, price: u64, signed_message: &AccountSignature) -> Result<OrderProcessingResult, AccountError> {
        signed_message.verify(&TestDiemCrypto(DUMMY_MESSAGE.to_string()), &account_pub_key).unwrap();
        self.place_limit_order(bank_controller, account_pub_key, side, qty, price)
    }

    // TODO - can we guard this to only be accessible in "bench" mode?
    // e.g. like #[cfg(bench)], except this only works locally
    pub fn overwrite_orderbook(&mut self, new_orderbook: Orderbook) {
        self.orderbook = new_orderbook;
    }
}
