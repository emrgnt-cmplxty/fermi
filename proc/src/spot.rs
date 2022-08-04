//!
//! this controller is responsible for managing interactions with a single orderbook
//! it relies on BankController to verify correctness of balances
//!
//! TODO
//! 0.) ADD MARKET ORDER SUPPORT
//! 2.) RESTRICT overwrite_orderbook TO BENCH ONLY MODE
//! 3.) CONSIDER ADDITIONAL FEATURES, LIKE ESCROW IMPLEMENTATION OR ORDER LIMITS
//! 4.) CHECK PASSED ASSETS EXIST IN BANK MODULE
//!
extern crate core;
extern crate engine;
extern crate types;

use super::{account::OrderAccount, bank::BankController};
use engine::{order_book::Orderbook, orders::new_limit_order_request};
use std::{collections::HashMap, time::SystemTime};
use types::{
    AccountPubKey, AssetId, AssetPairKey, OrderProcessingResult, OrderSide, ProcError, Success,
};

pub type OrderId = u64;

// The spot controller is responsible for accessing & modifying user orders
pub struct OrderbookInterface {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    orderbook: Orderbook,
    accounts: HashMap<AccountPubKey, OrderAccount>,
    order_to_account: HashMap<OrderId, AccountPubKey>,
}
impl OrderbookInterface {
    // TODO #4 //
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook = Orderbook::new(base_asset_id, quote_asset_id);
        OrderbookInterface {
            base_asset_id,
            quote_asset_id,
            orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), ProcError> {
        if self.accounts.contains_key(account_pub_key) {
            Err(ProcError::AccountCreation)
        } else {
            self.accounts.insert(
                account_pub_key.clone(),
                OrderAccount::new(account_pub_key.clone()),
            );
            Ok(())
        }
    }

    pub fn get_account(&self, account_pub_key: &AccountPubKey) -> Result<&OrderAccount, ProcError> {
        let account = self
            .accounts
            .get(account_pub_key)
            .ok_or(ProcError::AccountLookup)?;
        Ok(account)
    }

    pub fn place_limit_order(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        quantity: u64,
        price: u64,
    ) -> Result<OrderProcessingResult, ProcError> {
        // for now the orderbook creates an account if it is missing
        // in the future more robust handling is necessary to protect
        // the blockchain from spam
        if !self.accounts.contains_key(account_pub_key) {
            self.create_account(account_pub_key)?
        }

        let balance = *bank_controller.get_balance(account_pub_key, self.base_asset_id)?;
        // check balances before placing order
        if matches!(side, OrderSide::Ask) {
            // if ask, selling quantity of base asset
            assert!(balance > quantity);
        } else {
            // if bid, buying base asset with quantity*price of quote asset
            assert!(balance > quantity * price);
        }
        // create and process limit order
        let order = new_limit_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            side,
            price,
            quantity,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.proc_limit_result(bank_controller, account_pub_key, side, price, quantity, res)
    }

    // loop over and process the output from placing a limit order
    fn proc_limit_result(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        sub_side: OrderSide,
        sub_price: u64,
        sub_qty: u64,
        res: OrderProcessingResult,
    ) -> Result<OrderProcessingResult, ProcError> {
        for order in &res {
            match order {
                // first order is expected to be an Accepted result
                Ok(Success::Accepted { order_id, .. }) => {
                    self.proc_order_init(
                        bank_controller,
                        account_pub_key,
                        sub_side,
                        sub_price,
                        sub_qty,
                    )?;
                    // insert new order to map
                    self.order_to_account
                        .insert(*order_id, account_pub_key.clone());
                }
                // subsequent orders are expected to be an PartialFill or Fill results
                Ok(Success::PartiallyFilled {
                    order_id,
                    side,
                    price,
                    quantity,
                    ..
                }) => {
                    let existing_pub_key = self
                        .order_to_account
                        .get(order_id)
                        .ok_or(ProcError::AccountLookup)?
                        .clone();
                    self.proc_order_fill(
                        bank_controller,
                        &existing_pub_key,
                        *side,
                        *price,
                        *quantity,
                    )?;
                }
                Ok(Success::Filled {
                    order_id,
                    side,
                    price,
                    quantity,
                    ..
                }) => {
                    let existing_pub_key = self
                        .order_to_account
                        .get(order_id)
                        .ok_or(ProcError::AccountLookup)?
                        .clone();
                    self.proc_order_fill(
                        bank_controller,
                        &existing_pub_key,
                        *side,
                        *price,
                        *quantity,
                    )?;
                    // erase existing order
                    self.order_to_account
                        .remove(order_id)
                        .ok_or(ProcError::OrderRequest)?;
                }
                Ok(Success::Amended { .. }) => {
                    panic!("This needs to be implemented...")
                }
                Ok(Success::Cancelled { .. }) => {
                    panic!("This needs to be implemented...")
                }
                Err(_failure) => {
                    return Err(ProcError::OrderRequest);
                }
            }
        }
        Ok(res)
    }

    // process an initialized order by modifying the associated account
    fn proc_order_init(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), ProcError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. ask 1 BTC @ $20k moves 1 BTC (base) from balance to escrow
            bank_controller.update_balance(
                account_pub_key,
                self.base_asset_id,
                -(quantity as i64),
            )?;
        } else {
            // E.g. bid 1 BTC @ $20k moves 20k USD (quote) from balance to escrow
            bank_controller.update_balance(
                account_pub_key,
                self.quote_asset_id,
                -((quantity * price) as i64),
            )?;
        }
        Ok(())
    }

    // process a filled order by modifying the associated account
    fn proc_order_fill(
        &mut self,
        bank_controller: &mut BankController,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), ProcError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            bank_controller.update_balance(
                account_pub_key,
                self.quote_asset_id,
                (quantity * price) as i64,
            )?;
        } else {
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            bank_controller.update_balance(account_pub_key, self.base_asset_id, quantity as i64)?;
        }
        Ok(())
    }

    // TODO #2 //
    pub fn overwrite_orderbook(&mut self, new_orderbook: Orderbook) {
        self.orderbook = new_orderbook;
    }
}
pub struct SpotController {
    orderbooks: HashMap<AssetPairKey, OrderbookInterface>,
}
impl SpotController {
    pub fn new() -> Self {
        SpotController {
            orderbooks: HashMap::new(),
        }
    }

    fn get_orderbook_key(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> AssetPairKey {
        format!("{}_{}", base_asset_id, quote_asset_id)
    }

    // fn get_orderbook(
    //     &mut self,
    //     base_asset_id: AssetId,
    //     quote_asset_id: AssetId,
    // ) -> Result<&mut OrderbookInterface, ProcError> {
    //     let orderbook_lookup = self.get_orderbook_key(base_asset_id, quote_asset_id);

    //     let orderbook = self
    //         .orderbooks
    //         .get_mut(&orderbook_lookup)
    //         .ok_or_else(|| ProcError::AccountLookup)?;
    //     Ok(orderbook)
    // }

    pub fn create_orderbook(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
    ) -> Result<(), ProcError> {
        let orderbook_lookup = self.get_orderbook_key(base_asset_id, quote_asset_id);
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.orderbooks.entry(orderbook_lookup)
        {
            e.insert(OrderbookInterface::new(base_asset_id, quote_asset_id));
            Ok(())
        } else {
            Err(ProcError::OrderBookCreation)
        }
    }

    // pub fn parse_limit_order_transaction(
    //     &mut self,
    //     bank_controller: &mut BankController,
    //     signed_transaction: &TransactionRequest<TransactionVariant>,
    // ) -> Result<OrderProcessingResult, ProcError> {
    //     // verify transaction is an order
    //     if let TransactionVariant::OrderTransaction(order) = &signed_transaction.get_transaction() {
    //         // verify and place a limit order
    //         if let OrderRequest::Limit {
    //             side,
    //             price,
    //             quantity,
    //             base_asset,
    //             quote_asset,
    //             ..
    //         } = order
    //         {
    //             let orderbook: &mut OrderbookInterface = self.get_orderbook(*base_asset, *quote_asset)?;

    //             return orderbook.place_limit_order(
    //                 bank_controller,
    //                 signed_transaction.get_sender(),
    //                 *side,
    //                 *quantity,
    //                 *price,
    //             );
    //         } else {
    //             return Err(ProcError::OrderProc("Only limit orders supported".to_string()));
    //         }
    //     }
    //     Err(ProcError::OrderProc(
    //         "Only order transactions are supported".to_string(),
    //     ))
    // }
}

impl Default for SpotController {
    fn default() -> Self {
        Self::new()
    }
}
