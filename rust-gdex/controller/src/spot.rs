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
    order_book::{OrderBookWrapper, OrderId, Orderbook},
    orders::{create_cancel_order_request, create_limit_order_request, create_update_order_request},
};
use gdex_types::{
    account::{AccountPubKey, OrderAccount},
    asset::{AssetId, AssetPairKey},
    crypto::ToFromBytes,
    error::GDEXError,
    order_book::{OrderProcessingResult, OrderSide, OrderbookDepth},
    store::ProcessBlockStore,
    transaction::{
        deserialize_protobuf, parse_order_side, parse_request_type, CancelOrderRequest, CreateOrderbookRequest,
        LimitOrderRequest, MarketOrderRequest, RequestType, Transaction, UpdateOrderRequest,
    },
};

// external
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, time::SystemTime};

// CONSTANTS

pub const SPOT_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"SPOTCONTROLLERAAAAAAAAAAAAAAAAAA";
const ORDERBOOK_DEPTH_FREQUENCY: u64 = 100;

// ORDER BOOK INTERFACE

/// Creates a single orderbook instance and verifies all interactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpotInterface {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    controller_account: AccountPubKey,
    bank_controller: Arc<Mutex<BankController>>,
    orderbook: Orderbook,
    accounts: HashMap<AccountPubKey, OrderAccount>,
    order_to_account: HashMap<OrderId, AccountPubKey>,
}
// TODO - remove all asserts from orderbook impl
impl SpotInterface {
    // TODO #4 //
    pub fn new(
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        controller_account: AccountPubKey,
        bank_controller: Arc<Mutex<BankController>>,
    ) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook = Orderbook::new(base_asset_id, quote_asset_id);
        SpotInterface {
            base_asset_id,
            quote_asset_id,
            controller_account,
            bank_controller,
            orderbook,
            accounts: HashMap::new(),
            order_to_account: HashMap::new(),
        }
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

    fn get_base_asset_balance(&self, account: &AccountPubKey) -> Result<u64, GDEXError> {
        self.bank_controller
            .lock()
            .unwrap()
            .get_balance(account, self.base_asset_id)
    }

    fn get_quote_asset_balance(&self, account: &AccountPubKey) -> Result<u64, GDEXError> {
        self.bank_controller
            .lock()
            .unwrap()
            .get_balance(account, self.quote_asset_id)
    }

    fn validate_order_quantity(
        &self,
        account: &AccountPubKey,
        side: OrderSide,
        quantity: u64,
        price: u64,
        previous_quantity: u64,
        previous_price: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            // if ask, selling quantity of base asset
            if previous_quantity < quantity {
                let base_asset_balance = self.get_base_asset_balance(account)?;
                if base_asset_balance < (quantity - previous_quantity) {
                    return Err(GDEXError::OrderExceedsBalance);
                }
            }
        } else {
            // if bid, buying base asset with quantity*price of quote asset
            if previous_quantity * previous_price < quantity * previous_price {
                let quote_asset_balance = self.get_quote_asset_balance(account)?;
                if quote_asset_balance < (quantity * price - previous_quantity * previous_price) {
                    return Err(GDEXError::OrderExceedsBalance);
                }
            }
        }
        Ok(())
    }

    pub fn get_orderbook_depth(&self) -> OrderbookDepth {
        self.orderbook.get_orderbook_depth()
    }

    // TODO #2 //
    pub fn overwrite_orderbook(&mut self, new_orderbook: Orderbook) {
        self.orderbook = new_orderbook;
    }
}

impl OrderBookWrapper for SpotInterface {
    fn insert_new_order(&mut self, order_id: OrderId, account_pub_key: AccountPubKey) {
        self.order_to_account.insert(order_id, account_pub_key);
    }

    fn get_pub_key_from_order_id(&self, order_id: &OrderId) -> AccountPubKey {
        self.order_to_account
            .get(order_id)
            .ok_or(GDEXError::AccountLookup)
            .unwrap()
            .clone()
    }

    fn place_market_order(&mut self, _account: &AccountPubKey, _request: &MarketOrderRequest) -> Result<(), GDEXError> {
        Ok(())
    }

    // PLACE LIMIT ORDER

    fn place_limit_order(
        &mut self,
        account: &AccountPubKey,
        request: &LimitOrderRequest,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // create account
        if !self.accounts.contains_key(account) {
            self.create_account(account)?
        }

        // parse side
        let side = parse_order_side(request.side)?;

        // check balances before placing order
        self.validate_order_quantity(account, side, request.quantity, request.price, 0, 0)?;

        // create and process limit order
        let order = create_limit_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            side,
            request.price,
            request.quantity,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.process_order_result(account, res)
    }

    // PLACE CANCEL ORDER

    fn place_cancel_order(
        &mut self,
        account: &AccountPubKey,
        request: &CancelOrderRequest,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // create account
        if !self.accounts.contains_key(account) {
            self.create_account(account)?
        }

        // parse side
        let side = parse_order_side(request.side)?;

        // create and process limit order
        let order = create_cancel_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            request.order_id,
            side,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.process_order_result(account, res)
    }

    // PLACE UPDATE ORDER

    fn place_update_order(
        &mut self,
        account: &AccountPubKey,
        request: &UpdateOrderRequest,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // create account
        if !self.accounts.contains_key(account) {
            self.create_account(account)?
        }

        // parse side
        let side = parse_order_side(request.side)?;

        // check updates against user's balances
        let current_order = self.orderbook.get_order(side, request.order_id).unwrap();
        let current_quantity = current_order.get_quantity();
        let current_price = current_order.get_price();

        // check balances before placing order
        self.validate_order_quantity(
            account,
            side,
            request.quantity - current_quantity,
            request.price,
            current_quantity,
            current_price,
        )?;

        // create and process limit order
        let order = create_update_order_request(
            self.base_asset_id,
            self.quote_asset_id,
            request.order_id,
            side,
            request.price,
            request.quantity,
            SystemTime::now(),
        );
        let res = self.orderbook.process_order(order);
        self.process_order_result(account, res)
    }
    /// Processes an initialized order by modifying the associated account
    fn update_state_on_limit_order_creation(
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
    fn update_state_on_fill(
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

    fn update_state_on_update(
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

    fn update_state_on_cancel(
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
}
// INTERFACE

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpotController {
    controller_account: AccountPubKey,
    orderbooks: HashMap<AssetPairKey, SpotInterface>,
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

#[async_trait]
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
        let sender = transaction.get_sender()?;
        let request_type = parse_request_type(transaction.request_type)?;
        match request_type {
            RequestType::CreateOrderbook => {
                let request: CreateOrderbookRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.create_orderbook(request.base_asset_id, request.quote_asset_id)
            }
            RequestType::MarketOrder => {
                let request: MarketOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_market_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            RequestType::LimitOrder => {
                let request: LimitOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_limit_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            RequestType::UpdateOrder => {
                let request: UpdateOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_update_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            RequestType::CancelOrder => {
                let request: CancelOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_cancel_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            _ => Err(GDEXError::InvalidRequestTypeError),
        }
    }

    async fn process_end_of_block(
        controller: Arc<Mutex<Self>>,
        process_block_store: &ProcessBlockStore,
        block_number: u64,
    ) {
        // write out orderbook depth every ORDERBOOK_DEPTH_FREQUENCY
        if block_number % ORDERBOOK_DEPTH_FREQUENCY == 0 {
            let orderbook_depths = controller.lock().unwrap().generate_orderbook_depths();
            for (asset_pair, orderbook_depth) in orderbook_depths {
                process_block_store
                    .latest_orderbook_depth_store
                    .write(asset_pair, orderbook_depth.clone())
                    .await;
            }
        }
    }
}

impl SpotController {
    // Gets the order book key for a pair of assets
    pub fn get_orderbook_key(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> AssetPairKey {
        format!("{}_{}", base_asset_id, quote_asset_id)
    }

    // check if the orderbook has been created
    pub fn check_orderbook_exists(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> bool {
        let lookup_string = self.get_orderbook_key(base_asset_id, quote_asset_id);
        self.orderbooks.contains_key(&lookup_string)
    }

    pub fn generate_orderbook_depths(&self) -> HashMap<AssetPairKey, OrderbookDepth> {
        let mut orderbook_depths: HashMap<AssetPairKey, OrderbookDepth> = HashMap::new();
        for (asset_pair, orderbook) in &self.orderbooks {
            orderbook_depths.insert(asset_pair.clone(), orderbook.get_orderbook_depth());
        }

        orderbook_depths
    }

    pub fn create_orderbook(&mut self, base_asset_id: AssetId, quote_asset_id: AssetId) -> Result<(), GDEXError> {
        let lookup_string = self.get_orderbook_key(base_asset_id, quote_asset_id);
        if !self.check_orderbook_exists(base_asset_id, quote_asset_id) {
            self.orderbooks.insert(
                lookup_string,
                SpotInterface::new(
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
    ) -> Result<&mut SpotInterface, GDEXError> {
        let lookup_string = self.get_orderbook_key(base_asset_id, quote_asset_id);
        self.orderbooks.get_mut(&lookup_string).ok_or(GDEXError::AccountLookup)
    }
}

// TESTS

#[cfg(test)]
pub mod spot_tests {
    // crate
    use super::*;
    use crate::{
        bank::{BankController, CREATED_ASSET_BALANCE},
        spot::SpotInterface,
    };

    // gdex
    use gdex_types::crypto::KeypairTraits;
    use gdex_types::{
        account::account_test_functions::generate_keypair_vec,
        block::BlockDigest,
        order_book::{OrderSide, Success},
        transaction::{
            create_cancel_order_request, create_limit_order_request, create_limit_order_transaction,
            create_update_order_request,
        },
    };

    // mysten
    use fastcrypto::DIGEST_LEN;

    // constants

    const BASE_ASSET_ID: AssetId = 0;
    const QUOTE_ASSET_ID: AssetId = 1;
    const TRANSFER_AMOUNT: u64 = 1_000_000;

    // test helpers

    fn place_limit_order_helper(
        orderbook_interface: &mut SpotInterface,
        account: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> OrderProcessingResult {
        let limit_order_request =
            create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, side as u64, price, quantity);
        orderbook_interface
            .place_limit_order(account, &limit_order_request)
            .unwrap()
    }

    fn place_update_order_helper(
        orderbook_interface: &mut SpotInterface,
        account: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
        order_id: u64,
    ) -> OrderProcessingResult {
        let update_order_request =
            create_update_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, side as u64, price, quantity, order_id);
        orderbook_interface
            .place_update_order(account, &update_order_request)
            .unwrap()
    }

    fn place_cancel_order_helper(
        orderbook_interface: &mut SpotInterface,
        account: &AccountPubKey,
        side: OrderSide,
        order_id: u64,
    ) -> OrderProcessingResult {
        let cancel_order_request = create_cancel_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, side as u64, order_id);
        orderbook_interface
            .place_cancel_order(account, &cancel_order_request)
            .unwrap()
    }

    #[test]
    fn place_bid() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account.public(),
            OrderSide::Bid,
            bid_price,
            bid_size,
        );

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

        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let fee = 1000;
        let bid_size = 100;
        let bid_price = 100;
        let transaction = create_limit_order_transaction(
            account.public().clone(),
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid as u64,
            bid_price,
            bid_size,
            fee,
            recent_block_hash,
        );
        master_controller
            .spot_controller
            .lock()
            .unwrap()
            .handle_consensus_transaction(&transaction)
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

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account.public(),
            OrderSide::Ask,
            bid_price,
            bid_size,
        );

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
    fn fail_on_account_double_creation() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller: BankController = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotInterface::new(
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

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size_0: u64 = 100;
        let bid_price_0: u64 = 100;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account_0.public(),
            OrderSide::Bid,
            bid_price_0,
            bid_size_0,
        );

        let bid_size_1: u64 = 110;
        let bid_price_1: u64 = 110;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account_1.public(),
            OrderSide::Bid,
            bid_price_1,
            bid_size_1,
        );

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

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size_0: u64 = 95;
        let bid_price_0: u64 = 200;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account_0.public(),
            OrderSide::Bid,
            bid_price_0,
            bid_size_0,
        );

        let bid_size_1: u64 = bid_size_0;
        let bid_price_1: u64 = bid_price_0 - 2;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account_1.public(),
            OrderSide::Bid,
            bid_price_1,
            bid_size_1,
        );

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
        place_limit_order_helper(
            &mut orderbook_interface,
            &account_1.public(),
            OrderSide::Ask,
            ask_price_0,
            ask_size_0,
        );

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
        place_limit_order_helper(
            &mut orderbook_interface,
            &account_1.public(),
            OrderSide::Ask,
            ask_price_1,
            ask_size_1,
        );

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

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        let result = place_limit_order_helper(
            &mut orderbook_interface,
            &account.public(),
            OrderSide::Bid,
            bid_price,
            bid_size,
        );

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
            place_cancel_order_helper(&mut orderbook_interface, &account.public(), side, order_id);

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

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        const TEST_QUANTITY: u64 = 100;
        const TEST_PRICE: u64 = 100;
        const TEST_SIDE: OrderSide = OrderSide::Bid;
        let result = place_limit_order_helper(
            &mut orderbook_interface,
            &account.public(),
            TEST_SIDE,
            TEST_PRICE,
            TEST_QUANTITY,
        );

        if let Ok(Success::Accepted { order_id, side, .. }) = result[0] {
            // update order
            place_update_order_helper(
                &mut orderbook_interface,
                &account.public(),
                side,
                TEST_PRICE,
                TEST_QUANTITY + 1,
                order_id,
            );

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
    #[test]
    fn get_orderbook_depth() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();

        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotInterface::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
        );

        let bid_size = 100;
        let bid_price = 100;
        place_limit_order_helper(
            &mut orderbook_interface,
            &account.public(),
            OrderSide::Bid,
            bid_price,
            bid_size,
        );

        const TEST_MID: u64 = 100;
        const TEST_NUM_ORDERS: u64 = 2;

        for i in 1..3 {
            for _ in 0..TEST_NUM_ORDERS {
                place_limit_order_helper(
                    &mut orderbook_interface,
                    &account.public(),
                    OrderSide::Bid,
                    TEST_MID - i,
                    u64::pow(10 - i, 2),
                );
                place_limit_order_helper(
                    &mut orderbook_interface,
                    &account.public(),
                    OrderSide::Ask,
                    TEST_MID + i,
                    u64::pow(10 - i, 2),
                );
            }
        }
        let _orderbook_depth = orderbook_interface.get_orderbook_depth();
    }
}
