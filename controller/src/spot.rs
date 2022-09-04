//! Creates orderbooks and manages their interactions
//!
//! TODO
//! 0.) ADD MARKET ORDER SUPPORT
//! 2.) RESTRICT overwrite_orderbook TO BENCH ONLY MODE
//! 3.) CONSIDER ADDITIONAL FEATURES, LIKE ESCROW IMPLEMENTATION OR ORDER LIMITS
//! 4.) CHECK PASSED ASSETS EXIST IN BANK MODULE
//!
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// IMPORTS

// crate
use super::bank::BankController;
use crate::controller::Controller;
use crate::master::MasterController;

// gdex
use gdex_engine::{
    order_book::Orderbook,
    orders::{create_cancel_order_request, create_limit_order_request, create_update_order_request},
};
use gdex_types::{
    account::{AccountPubKey, OrderAccount},
    asset::{AssetId, AssetPairKey},
    crypto::ToFromBytes,
    error::GDEXError,
    order_book::{OrderProcessingResult, OrderSide, OrderType, Success},
    transaction::{OrderRequest, Transaction, TransactionVariant},
};

// mysten
use fastcrypto::ed25519::Ed25519PublicKey;

// external
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, time::SystemTime};

// TYPE DEFS

pub type OrderId = u64;

// CONSTANTS

pub const SPOT_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"SPOTCONTROLLERAAAAAAAAAAAAAAAAAA";

// ORDER BOOK INTERFACE

/// Creates a single orderbook instance and verifies all interactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderbookInterface {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    controller_account: AccountPubKey,
    bank_controller: Arc<Mutex<BankController>>,
    orderbook: Orderbook,
    accounts: HashMap<AccountPubKey, OrderAccount>,
    order_to_account: HashMap<OrderId, AccountPubKey>,
}
// TODO - remove all asserts from orderbook impl
impl OrderbookInterface {
    // TODO #4 //
    pub fn new(
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        controller_account: AccountPubKey,
        bank_controller: Arc<Mutex<BankController>>,
    ) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook = Orderbook::new(base_asset_id, quote_asset_id);
        OrderbookInterface {
            base_asset_id,
            quote_asset_id,
            controller_account,
            bank_controller,
            orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
        }
    }

    fn get_pub_key_from_order(&self, order_id: &OrderId) -> Ed25519PublicKey {
        self.order_to_account
            .get(order_id)
            .ok_or(GDEXError::AccountLookup)
            .unwrap()
            .clone()
    }

    /// Create a new account in the orderbook
    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), GDEXError> {
        if self.accounts.contains_key(account_pub_key) {
            Err(GDEXError::AccountCreation)
        } else {
            self.accounts
                .insert(account_pub_key.clone(), OrderAccount::new(account_pub_key.clone()));
            Ok(())
        }
    }

    /// Get an account in the orderbook
    pub fn get_account(&self, account_pub_key: &AccountPubKey) -> Result<&OrderAccount, GDEXError> {
        let account = self.accounts.get(account_pub_key).ok_or(GDEXError::AccountLookup)?;
        Ok(account)
    }

    /// Attempt to place a limit order into the orderbook
    pub fn place_limit_order(
        &mut self,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        quantity: u64,
        price: u64,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // for now the orderbook creates an account if it is missing
        // in the future more robust handling is necessary to protect
        // the blockchain from spam
        if !self.accounts.contains_key(account_pub_key) {
            self.create_account(account_pub_key)?
        }

        // check balances before placing order
        if matches!(side, OrderSide::Ask) {
            // if ask, selling quantity of base asset
            let base_asset_balance = self
                .bank_controller
                .lock()
                .unwrap()
                .get_balance(account_pub_key, self.base_asset_id)?;
            if base_asset_balance < quantity {
                return Err(GDEXError::OrderExceedsBalance);
            }
        } else {
            // if bid, buying base asset with quantity*price of quote asset
            let quote_asset_balance = self
                .bank_controller
                .lock()
                .unwrap()
                .get_balance(account_pub_key, self.quote_asset_id)?;

            if quote_asset_balance < quantity * price {
                return Err(GDEXError::OrderExceedsBalance);
            }
        }

        // create and process limit order
        let order = create_limit_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            side,
            price,
            quantity,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.process_order_result(account_pub_key, res)
    }

    /// Attempt to place a limit order into the orderbook
    pub fn place_cancel_order(
        &mut self,
        account_pub_key: &AccountPubKey,
        order_id: OrderId,
        side: OrderSide,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // create account
        if !self.accounts.contains_key(account_pub_key) {
            self.create_account(account_pub_key)?
        }

        // create and process limit order
        let order = create_cancel_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            order_id,
            side,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.process_order_result(account_pub_key, res)
    }

    /// Attempt to place a limit order into the orderbook
    pub fn place_update_order(
        &mut self,
        account_pub_key: &AccountPubKey,
        order_id: OrderId,
        side: OrderSide,
        quantity: u64,
        price: u64,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // create account
        if !self.accounts.contains_key(account_pub_key) {
            self.create_account(account_pub_key)?
        }

        // check updates against user's balances
        let current_order = self.orderbook.get_order(side, order_id).unwrap();
        let current_quantity = current_order.get_quantity();
        let current_price = current_order.get_price();

        // check balances before placing order
        if matches!(side, OrderSide::Ask) {
            // if ask, selling quantity of base asset
            if quantity > current_quantity {
                let base_asset_balance = self
                    .bank_controller
                    .lock()
                    .unwrap()
                    .get_balance(account_pub_key, self.base_asset_id)?;

                assert!(base_asset_balance > quantity - current_quantity);
            }
        } else {
            // if bid, buying base asset with quantity*price of quote asset
            if quantity * price > current_quantity * current_price {
                let quote_asset_balance = self
                    .bank_controller
                    .lock()
                    .unwrap()
                    .get_balance(account_pub_key, self.quote_asset_id)?;

                assert!(quote_asset_balance > quantity * price - current_quantity * current_price);
            }
        }

        // create and process limit order
        let order = create_update_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            order_id,
            side,
            price,
            quantity,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.process_order_result(account_pub_key, res)
    }

    /// Attempts to loop over and process the outputs from a placed limit order
    fn process_order_result(
        &mut self,
        account_pub_key: &AccountPubKey,
        res: OrderProcessingResult,
    ) -> Result<OrderProcessingResult, GDEXError> {
        for order in &res {
            match order {
                // first order is expected to be an Accepted result
                Ok(Success::Accepted {
                    order_id,
                    side,
                    price,
                    quantity,
                    order_type,
                    ..
                }) => {
                    // update user's balances if it is a limit order
                    if *order_type == OrderType::Limit {
                        self.update_balances_on_limit_order_create(account_pub_key, *side, *price, *quantity)?;
                    }
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
                    // update user balances
                    let existing_pub_key = self.get_pub_key_from_order(order_id);
                    self.update_balances_on_fill(&existing_pub_key, *side, *price, *quantity)?;
                }
                Ok(Success::Filled {
                    order_id,
                    side,
                    price,
                    quantity,
                    ..
                }) => {
                    // update user balances
                    let existing_pub_key = self.get_pub_key_from_order(order_id);
                    self.update_balances_on_fill(&existing_pub_key, *side, *price, *quantity)?;
                    // remove order from map
                    self.order_to_account.remove(order_id).ok_or(GDEXError::OrderRequest)?;
                }
                Ok(Success::Updated {
                    order_id,
                    side,
                    previous_price,
                    previous_quantity,
                    price,
                    quantity,
                    ..
                }) => {
                    let existing_pub_key = self.get_pub_key_from_order(order_id);
                    dbg!(*previous_price, *previous_quantity, *price, *quantity);
                    self.update_balances_on_update(
                        &existing_pub_key,
                        *side,
                        *previous_price,
                        *previous_quantity,
                        *price,
                        *quantity,
                    )?;
                }
                Ok(Success::Cancelled {
                    order_id,
                    side,
                    price,
                    quantity,
                    ..
                }) => {
                    // order has been cancelled from order book, update states
                    let existing_pub_key = self.get_pub_key_from_order(order_id);
                    self.update_balances_on_cancel(&existing_pub_key, *side, *price, *quantity)?;
                }
                Err(_failure) => {
                    return Err(GDEXError::OrderRequest);
                }
            }
        }
        Ok(res)
    }

    /// Processes an initialized order by modifying the associated account
    fn update_balances_on_limit_order_create(
        &mut self,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. ask 1 BTC @ $20k moves 1 BTC (base) from balance to escrow
            self.bank_controller.lock().unwrap().transfer(
                account_pub_key,
                &self.controller_account,
                self.base_asset_id,
                quantity,
            )?;
        } else {
            // E.g. bid 1 BTC @ $20k moves 20k USD (quote) from balance to escrow
            self.bank_controller.lock().unwrap().transfer(
                account_pub_key,
                &self.controller_account,
                self.quote_asset_id,
                quantity * price,
            )?;
        }
        Ok(())
    }

    /// Processes a filled order by modifying the associated account
    fn update_balances_on_fill(
        &mut self,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            self.bank_controller.lock().unwrap().transfer(
                &self.controller_account,
                account_pub_key,
                self.quote_asset_id,
                quantity * price,
            )?;
        } else {
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            self.bank_controller.lock().unwrap().transfer(
                &self.controller_account,
                account_pub_key,
                self.base_asset_id,
                quantity,
            )?;
        }
        Ok(())
    }

    fn update_balances_on_update(
        &mut self,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        previous_price: u64,
        previous_quantity: u64,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            // E.g. fill ask 1 BTC @ 20k adds 20k USD (quote) to bal, subtracts 1 BTC (base) from escrow
            if quantity > previous_quantity {
                self.bank_controller.lock().unwrap().transfer(
                    account_pub_key,
                    &self.controller_account,
                    self.base_asset_id,
                    quantity - previous_quantity,
                )?;
            } else {
                self.bank_controller.lock().unwrap().transfer(
                    &self.controller_account,
                    account_pub_key,
                    self.base_asset_id,
                    previous_quantity - quantity,
                )?;
            }
        } else {
            // E.g. fill bid 1 BTC @ 20k adds 1 BTC (base) to bal, subtracts 20k USD (quote) from escrow
            if quantity * price > previous_quantity * previous_price {
                dbg!(quantity * price - previous_quantity * previous_price);
                self.bank_controller.lock().unwrap().transfer(
                    account_pub_key,
                    &self.controller_account,
                    self.quote_asset_id,
                    quantity * price - previous_quantity * previous_price,
                )?;
            } else {
                self.bank_controller.lock().unwrap().transfer(
                    &self.controller_account,
                    account_pub_key,
                    self.quote_asset_id,
                    previous_quantity * previous_price - quantity * price,
                )?;
            }
        }
        Ok(())
    }

    fn update_balances_on_cancel(
        &mut self,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            self.bank_controller.lock().unwrap().transfer(
                &self.controller_account,
                account_pub_key,
                self.base_asset_id,
                quantity,
            )?;
        } else {
            self.bank_controller.lock().unwrap().transfer(
                &self.controller_account,
                account_pub_key,
                self.quote_asset_id,
                quantity * price,
            )?;
        }
        Ok(())
    }

    // TODO #2 //
    pub fn overwrite_orderbook(&mut self, new_orderbook: Orderbook) {
        self.orderbook = new_orderbook;
    }
}

// INTERFACE

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpotController {
    controller_account: AccountPubKey,
    orderbooks: HashMap<AssetPairKey, OrderbookInterface>,
    bank_controller: Arc<Mutex<BankController>>,
}

impl Default for SpotController {
    fn default() -> Self {
        Self {
            controller_account: AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            orderbooks: HashMap::new(),
            bank_controller: Arc::new(Mutex::new(BankController::default())), // TEMPORARY
        }
    }
}

impl Controller for SpotController {
    fn initialize(&mut self, master_controller: &MasterController) {
        self.bank_controller = Arc::clone(&master_controller.bank_controller);
    }

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError> {
        self.bank_controller
            .lock()
            .unwrap()
            .create_account(&self.controller_account)?;
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError> {
        if let TransactionVariant::CreateOrderbookTransaction(orderbook) = transaction.get_variant() {
            return self.create_orderbook(orderbook.get_base_asset_id(), orderbook.get_quote_asset_id());
        }
        if let TransactionVariant::PlaceOrderTransaction(order) = transaction.get_variant() {
            match order {
                OrderRequest::Market {
                    base_asset_id,
                    quote_asset_id,
                    side,
                    quantity,
                    ..
                } => {
                    dbg!(base_asset_id, quote_asset_id, side, quantity);
                }
                OrderRequest::Limit {
                    base_asset_id,
                    quote_asset_id,
                    side,
                    price,
                    quantity,
                    ..
                } => self.place_limit_order(
                    *base_asset_id,
                    *quote_asset_id,
                    transaction.get_sender(),
                    *side,
                    *quantity,
                    *price,
                )?,
                OrderRequest::Cancel {
                    base_asset_id,
                    quote_asset_id,
                    order_id,
                    side,
                    ..
                } => self.place_cancel_order(
                    *base_asset_id,
                    *quote_asset_id,
                    transaction.get_sender(),
                    *order_id,
                    *side,
                )?,
                OrderRequest::Update {
                    base_asset_id,
                    quote_asset_id,
                    order_id,
                    side,
                    price,
                    quantity,
                    ..
                } => self.place_update_order(
                    *base_asset_id,
                    *quote_asset_id,
                    transaction.get_sender(),
                    *order_id,
                    *side,
                    *quantity,
                    *price,
                )?,
            }
        }

        Ok(())
    }

    fn post_process(&mut self, _block_number: u64) {}
}

impl SpotController {
    // Gets the order book key for a pair of assets
    fn _get_orderbook_key(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> AssetPairKey {
        format!("{}_{}", base_asset_id, quote_asset_id)
    }

    // check if the orderbook has been created
    pub fn check_orderbook_exists(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> bool {
        let lookup_string = self._get_orderbook_key(base_asset_id, quote_asset_id);
        self.orderbooks.contains_key(&lookup_string)
    }

    pub fn create_orderbook(&mut self, base_asset_id: AssetId, quote_asset_id: AssetId) -> Result<(), GDEXError> {
        let lookup_string = self._get_orderbook_key(base_asset_id, quote_asset_id);
        if !self.check_orderbook_exists(base_asset_id, quote_asset_id) {
            self.orderbooks.insert(
                lookup_string,
                OrderbookInterface::new(
                    base_asset_id,
                    quote_asset_id,
                    self.controller_account.clone(),
                    Arc::clone(&self.bank_controller),
                ),
            );
            Ok(())
        } else {
            Err(GDEXError::OrderBookCreation)
        }
    }

    // Attempts to retrieve an order book from the controller
    pub fn get_orderbook(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
    ) -> Result<&mut OrderbookInterface, GDEXError> {
        let lookup_string = self._get_orderbook_key(base_asset_id, quote_asset_id);
        self.orderbooks.get_mut(&lookup_string).ok_or(GDEXError::AccountLookup)
    }

    /// Attempts to get an order book and places a limit order
    pub fn place_limit_order(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        account_pub_key: &AccountPubKey,
        side: OrderSide,
        quantity: u64,
        price: u64,
    ) -> Result<(), GDEXError> {
        match self.get_orderbook(base_asset_id, quote_asset_id)?.place_limit_order(
            account_pub_key,
            side,
            quantity,
            price,
        ) {
            Ok(_ordering_processing_result) => Ok(()),
            Err(_err) => Err(GDEXError::OrderRequest),
        }
    }

    pub fn place_cancel_order(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        account_pub_key: &AccountPubKey,
        order_id: OrderId,
        side: OrderSide,
    ) -> Result<(), GDEXError> {
        match self
            .get_orderbook(base_asset_id, quote_asset_id)?
            .place_cancel_order(account_pub_key, order_id, side)
        {
            Ok(_ordering_processing_result) => Ok(()),
            Err(_err) => Err(GDEXError::OrderRequest),
        }
    }

    /// Attempts to get an order book and places a limit order
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    pub fn place_update_order(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        account_pub_key: &AccountPubKey,
        order_id: OrderId,
        side: OrderSide,
        quantity: u64,
        price: u64,
    ) -> Result<(), GDEXError> {
        match self.get_orderbook(base_asset_id, quote_asset_id)?.place_update_order(
            account_pub_key,
            order_id,
            side,
            quantity,
            price,
        ) {
            Ok(_ordering_processing_result) => Ok(()),
            Err(_err) => Err(GDEXError::OrderRequest),
        }
    }
}

#[cfg(test)]
pub mod spot_tests {
    use super::*;
    use crate::{
        bank::{BankController, CREATED_ASSET_BALANCE},
        spot::OrderbookInterface,
    };
    use gdex_types::crypto::KeypairTraits;
    use gdex_types::{account::account_test_functions::generate_keypair_vec, order_book::OrderSide};

    const BASE_ASSET_ID: AssetId = 0;
    const QUOTE_ASSET_ID: AssetId = 1;
    const TRANSFER_AMOUNT: u64 = 1_000_000;

    #[test]
    fn place_bid() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        orderbook_interface
            .place_limit_order(account.public(), OrderSide::Bid, bid_size, bid_price)
            .unwrap();

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size * bid_price
        );
    }

    #[test]
    fn place_bid_spot_controller() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let master_controller = MasterController::default();
        master_controller.initialize_controllers();
        master_controller.initialize_controller_accounts();

        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(account.public())
            .unwrap();
        master_controller
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(account.public())
            .unwrap();

        master_controller
            .spot_controller
            .lock()
            .unwrap()
            .create_orderbook(BASE_ASSET_ID, QUOTE_ASSET_ID)
            .unwrap();

        let bid_size = 100;
        let bid_price = 100;
        master_controller
            .spot_controller
            .lock()
            .unwrap()
            .place_limit_order(
                BASE_ASSET_ID,
                QUOTE_ASSET_ID,
                account.public(),
                OrderSide::Bid,
                bid_size,
                bid_price,
            )
            .unwrap();

        let bank_controller_ref = Arc::clone(&master_controller.bank_controller);

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size * bid_price
        );
    }

    #[test]
    fn place_ask() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        orderbook_interface
            .place_limit_order(account.public(), OrderSide::Ask, bid_size, bid_price)
            .unwrap();

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size
        );
    }

    #[test]
    fn fail_on_invalid_account_lookup() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let result = orderbook_interface.get_account(account.public()).unwrap_err();

        assert!(matches!(result, GDEXError::AccountLookup));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        orderbook_interface.create_account(account.public()).unwrap();
        let result = orderbook_interface.create_account(account.public()).unwrap_err();
        assert!(matches!(result, GDEXError::AccountCreation));
    }

    #[test]
    fn multi_bid() {
        let account_0 = generate_keypair_vec([0; 32]).pop().unwrap();
        let account_1 = generate_keypair_vec([1; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::default();

        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), BASE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), QUOTE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();

        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

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
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT
        );
    }

    #[test]
    fn multi_bid_and_ask() {
        let account_0 = generate_keypair_vec([0; 32]).pop().unwrap();
        let account_1 = generate_keypair_vec([1; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::default();

        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), BASE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), QUOTE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();

        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

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
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
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
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
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
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
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
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT + bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );
    }

    #[test]
    fn place_cancel_order() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        let result = orderbook_interface
            .place_limit_order(account.public(), OrderSide::Bid, bid_size, bid_price)
            .unwrap();

        if let Ok(Success::Accepted { order_id, side, .. }) = result[0] {
            // get order
            assert!(
                orderbook_interface.orderbook.get_order(side, order_id).is_ok(),
                "Order has been created"
            );

            // check user balances
            let user_quote_balance = bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap();
            assert_eq!(user_quote_balance, CREATED_ASSET_BALANCE - 100 * 100);

            // cancel order
            orderbook_interface
                .place_cancel_order(account.public(), order_id, side)
                .unwrap();

            assert!(
                orderbook_interface.orderbook.get_order(side, order_id).is_err(),
                "Order has been cancelled"
            );

            // check user balances
            let user_quote_balance = bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap();
            assert_eq!(user_quote_balance, CREATED_ASSET_BALANCE);
        }
    }

    #[test]
    fn place_update() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = OrderbookInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        const TEST_QUANTITY: u64 = 100;
        const TEST_PRICE: u64 = 100;
        const TEST_SIDE: OrderSide = OrderSide::Bid;
        let result = orderbook_interface
            .place_limit_order(account.public(), TEST_SIDE, TEST_QUANTITY, TEST_PRICE)
            .unwrap();

        if let Ok(Success::Accepted { order_id, side, .. }) = result[0] {
            // update order
            orderbook_interface
                .place_update_order(account.public(), order_id, side, TEST_QUANTITY + 1, TEST_PRICE)
                .unwrap();

            assert!(
                orderbook_interface
                    .orderbook
                    .get_order(side, order_id)
                    .unwrap()
                    .get_price()
                    == TEST_PRICE,
                "Price did not change."
            );
            assert!(
                orderbook_interface
                    .orderbook
                    .get_order(side, order_id)
                    .unwrap()
                    .get_quantity()
                    == TEST_QUANTITY + 1,
                "Quantity did change."
            );

            // check user balances
            assert_eq!(
                bank_controller_ref
                    .lock()
                    .unwrap()
                    .get_balance(account.public(), BASE_ASSET_ID)
                    .unwrap(),
                CREATED_ASSET_BALANCE
            );
            assert_eq!(
                bank_controller_ref
                    .lock()
                    .unwrap()
                    .get_balance(account.public(), QUOTE_ASSET_ID)
                    .unwrap(),
                CREATED_ASSET_BALANCE - (TEST_QUANTITY + 1) * TEST_PRICE
            );
        }
    }
}
