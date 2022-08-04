//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
//! This controller is responsible for managing interactions with a single orderbook
//! it relies on BankController to verify correctness of balances
//!
//! TODO
//! 0.) ADD MARKET ORDER SUPPORT
//! 2.) RESTRICT overwrite_orderbook TO BENCH ONLY MODE
//! 3.) CONSIDER ADDITIONAL FEATURES, LIKE ESCROW IMPLEMENTATION OR ORDER LIMITS
//! 4.) CHECK PASSED ASSETS EXIST IN BANK MODULE
//!
//!
use super::bank::BankController;
use core::cell::RefCell;
use gdex_engine::{order_book::Orderbook, orders::new_limit_order_request};
use gdex_types::{AccountPubKey, AssetId, AssetPairKey, OrderAccount, OrderProcessingResult, OrderSide, ProcError, Success};
use std::{collections::HashMap, rc::Rc, time::SystemTime};

pub type OrderId = u64;

// The spot controller is responsible for accessing & modifying user orders
pub struct OrderbookInterface {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    orderbook: Orderbook,
    accounts: HashMap<AccountPubKey, OrderAccount>,
    order_to_account: HashMap<OrderId, AccountPubKey>,
    bank_controller: Rc<RefCell<BankController>>,
}
impl OrderbookInterface {
    // TODO #4 //
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId, bank_controller: Rc<RefCell<BankController>>) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook = Orderbook::new(base_asset_id, quote_asset_id);
        OrderbookInterface {
            base_asset_id,
            quote_asset_id,
            orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
            bank_controller,
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), ProcError> {
        if self.accounts.contains_key(account_pub_key) {
            Err(ProcError::AccountCreation)
        } else {
            self.accounts
                .insert(account_pub_key.clone(), OrderAccount::new(account_pub_key.clone()));
            Ok(())
        }
    }

    pub fn get_account(&self, account_pub_key: &AccountPubKey) -> Result<&OrderAccount, ProcError> {
        let account = self.accounts.get(account_pub_key).ok_or(ProcError::AccountLookup)?;
        Ok(account)
    }

    pub fn place_limit_order(
        &mut self,
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

        let balance = *self
            .bank_controller
            .borrow()
            .get_balance(account_pub_key, self.base_asset_id)?;
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
        self.proc_limit_result(account_pub_key, side, price, quantity, res)
    }

    // loop over and process the output from placing a limit order
    fn proc_limit_result(
        &mut self,
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
                    self.proc_order_init(account_pub_key, sub_side, sub_price, sub_qty)?;
                    // insert new order to map
                    self.order_to_account.insert(*order_id, account_pub_key.clone());
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
                    self.proc_order_fill(&existing_pub_key, *side, *price, *quantity)?;
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
                    self.proc_order_fill(&existing_pub_key, *side, *price, *quantity)?;
                    // erase existing order
                    self.order_to_account.remove(order_id).ok_or(ProcError::OrderRequest)?;
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
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), ProcError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. ask 1 BTC @ $20k moves 1 BTC (base) from balance to escrow
            self.bank_controller.borrow_mut().update_balance(
                account_pub_key,
                self.base_asset_id,
                -(quantity as i64),
            )?;
        } else {
            // E.g. bid 1 BTC @ $20k moves 20k USD (quote) from balance to escrow
            self.bank_controller.borrow_mut().update_balance(
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
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), ProcError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            self.bank_controller.borrow_mut().update_balance(
                account_pub_key,
                self.quote_asset_id,
                (quantity * price) as i64,
            )?;
        } else {
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            self.bank_controller
                .borrow_mut()
                .update_balance(account_pub_key, self.base_asset_id, quantity as i64)?;
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
    bank_controller: Rc<RefCell<BankController>>,
}
impl SpotController {
    pub fn new(bank_controller: Rc<RefCell<BankController>>) -> Self {
        SpotController {
            orderbooks: HashMap::new(),
            bank_controller,
        }
    }

    fn get_orderbook_key(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> AssetPairKey {
        format!("{}_{}", base_asset_id, quote_asset_id)
    }

    fn get_orderbook(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
    ) -> Result<&mut OrderbookInterface, ProcError> {
        let orderbook_lookup = self.get_orderbook_key(base_asset_id, quote_asset_id);

        let orderbook = self
            .orderbooks
            .get_mut(&orderbook_lookup)
            .ok_or(ProcError::AccountLookup)?;
        Ok(orderbook)
    }

    pub fn create_orderbook(&mut self, base_asset_id: AssetId, quote_asset_id: AssetId) -> Result<(), ProcError> {
        let orderbook_lookup = self.get_orderbook_key(base_asset_id, quote_asset_id);
        if let std::collections::hash_map::Entry::Vacant(e) = self.orderbooks.entry(orderbook_lookup) {
            e.insert(OrderbookInterface::new(
                base_asset_id,
                quote_asset_id,
                Rc::clone(&self.bank_controller),
            ));
            Ok(())
        } else {
            Err(ProcError::OrderBookCreation)
        }
    }

    // pub fn parse_limit_order_transaction(
    //     &mut self,
    //     signed_transaction: &SignedTransaction,
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

    #[cfg(test)]
    pub fn place_limit_order(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        quantity: u64,
        price: u64,
    ) -> Result<OrderProcessingResult, ProcError> {
        self.get_orderbook(base_asset_id, quote_asset_id)?
            .place_limit_order(account_pub_key, side, quantity, price)
    }
}

// impl Default for SpotController {
//     fn default() -> Self {
//         Self::new()
//     }
// }

#[cfg(test)]
pub mod spot_tests {
    use super::*;
    use crate::{
        bank::{BankController, CREATED_ASSET_BALANCE},
        spot::OrderbookInterface,
    };
    use gdex_types::{account_test_functions::generate_keypair_vec, OrderSide};
    use narwhal_crypto::traits::KeyPair;

    const BASE_ASSET_ID: AssetId = 0;
    const QUOTE_ASSET_ID: AssetId = 1;
    const TRANSFER_AMOUNT: u64 = 1_000_000;

    #[test]
    fn place_bid() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, Rc::clone(&bank_controller_ref));

        let bid_size = 100;
        let bid_price = 100;
        orderbook_interface
            .place_limit_order(account.public(), OrderSide::Bid, bid_size, bid_price)
            .unwrap();

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size * bid_price
        );
    }

    #[test]
    fn place_bid_spot_controller() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let mut spot_controller = SpotController::new(Rc::clone(&bank_controller_ref));

        spot_controller.create_orderbook(BASE_ASSET_ID, QUOTE_ASSET_ID).unwrap();

        let bid_size = 100;
        let bid_price = 100;
        spot_controller
            .place_limit_order(
                BASE_ASSET_ID,
                QUOTE_ASSET_ID,
                account.public(),
                OrderSide::Bid,
                bid_size,
                bid_price,
            )
            .unwrap();

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size * bid_price
        );
    }

    #[test]
    fn place_ask() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, Rc::clone(&bank_controller_ref));

        let bid_size = 100;
        let bid_price = 100;
        orderbook_interface
            .place_limit_order(account.public(), OrderSide::Ask, bid_size, bid_price)
            .unwrap();

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size
        );
    }

    #[test]
    fn fail_on_invalid_account_lookup() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::new();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, Rc::clone(&bank_controller_ref));

        let result = orderbook_interface.get_account(account.public()).unwrap_err();

        assert!(matches!(result, ProcError::AccountLookup));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, Rc::clone(&bank_controller_ref));

        orderbook_interface.create_account(account.public()).unwrap();
        let result = orderbook_interface.create_account(account.public()).unwrap_err();
        assert!(matches!(result, ProcError::AccountCreation));
    }

    #[test]
    fn multi_bid() {
        let account_0 = generate_keypair_vec([0; 32]).pop().unwrap();
        let account_1 = generate_keypair_vec([1; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::new();

        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), BASE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), QUOTE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();

        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, Rc::clone(&bank_controller_ref));

        let bid_size_0: u64 = 100;
        let bid_price_0: u64 = 100;
        orderbook_interface
            .place_limit_order(account_0.public(), OrderSide::Bid, bid_size_0, bid_price_0)
            .unwrap();

        let bid_size_1: u64 = 110;
        let bid_price_1: u64 = 110;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Bid, bid_size_1, bid_price_1)
            .unwrap();

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(&account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT
        );
    }

    #[test]
    fn multi_bid_and_ask() {
        let account_0 = generate_keypair_vec([0; 32]).pop().unwrap();
        let account_1 = generate_keypair_vec([1; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::new();

        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), BASE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), QUOTE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();

        let bank_controller_ref = Rc::new(RefCell::new(bank_controller));

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, Rc::clone(&bank_controller_ref));

        let bid_size_0: u64 = 95;
        let bid_price_0: u64 = 200;
        orderbook_interface
            .place_limit_order(account_0.public(), OrderSide::Bid, bid_size_0, bid_price_0)
            .unwrap();

        let bid_size_1: u64 = bid_size_0;
        let bid_price_1: u64 = bid_price_0 - 2;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Bid, bid_size_1, bid_price_1)
            .unwrap();

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT
        );

        // Place ask for account 1 at price that crosses spread entirely
        let ask_size_0: u64 = bid_size_0;
        let ask_price_0: u64 = bid_price_0 - 1;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Ask, ask_size_0, ask_price_0)
            .unwrap();

        // check account 0
        // received initial asset creation balance
        // paid bid_size_0 * bid_price_0 in quote asset to orderbook
        // received bid_size_0 in base asset from settled trade
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // received initial transfer amount
        // received bid_size_0 * bid_price_0 in quote asset to balance
        // sent bid_size_1 * bid_price_1 in quote asset to escrow
        // paid bid_size_0 in base asset from balance
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );

        // Place final order for account 1 at price that crosses spread entirely and closes it's own position
        let ask_size_1: u64 = bid_size_1;
        let ask_price_1: u64 = bid_price_1 - 1;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Ask, ask_size_1, ask_price_1)
            .unwrap();

        // check account 0
        // state should remain unchanged from prior
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT + bid_size_0 * bid_price_0
        );
        assert_eq!(
            *bank_controller_ref
                .borrow()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );
    }
}
