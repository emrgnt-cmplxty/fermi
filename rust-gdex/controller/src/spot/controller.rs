//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

// TODO - https://github.com/gdexorg/gdex/issues/171 - verify asset existence before use

// IMPORTS

// crate
use crate::bank::controller::BankController;
use crate::controller::Controller;
use crate::event_manager::{EventEmitter, EventManager};
use crate::router::ControllerRouter;
use crate::spot::proto::*;
use crate::utils::engine::order_book::{OrderBookWrapper, OrderId, Orderbook};

// gdex
use gdex_types::{
    account::AccountPubKey,
    asset::{AssetId, AssetPairKey},
    crypto::ToFromBytes,
    error::GDEXError,
    order_book::{OrderSide, OrderbookDepth},
    store::ProcessBlockStore,
    transaction::{deserialize_protobuf, Transaction},
};

// external
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// CONSTANTS

pub const SPOT_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"SPOTCONTROLLERAAAAAAAAAAAAAAAAAA";
const ORDERBOOK_DEPTH_FREQUENCY: u64 = 100;

// INTERFACE
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpotController {
    // controller state
    controller_account: AccountPubKey,
    orderbooks: HashMap<AssetPairKey, SpotOrderbook>,
    bank_controller: Arc<Mutex<BankController>>,
    // shared
    event_manager: Arc<Mutex<EventManager>>,
}

impl Default for SpotController {
    fn default() -> Self {
        Self {
            controller_account: AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            orderbooks: HashMap::new(),
            bank_controller: Arc::new(Mutex::new(BankController::default())), // TEMPORARY
            // shared state
            event_manager: Arc::new(Mutex::new(EventManager::new())), // TEMPORARY
        }
    }
}

#[async_trait]
impl Controller for SpotController {
    fn initialize(&mut self, controller_router: &ControllerRouter) {
        self.bank_controller = Arc::clone(&controller_router.bank_controller);
        self.event_manager = Arc::clone(&controller_router.event_manager);
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
        let request_type: SpotRequestType = transaction.get_request_type()?;
        match request_type {
            SpotRequestType::CreateOrderbook => {
                let request: CreateOrderbookRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.create_orderbook(request.base_asset_id, request.quote_asset_id)
            }
            // TODO - https://github.com/gdexorg/gdex/issues/170 - add support for market orders
            SpotRequestType::MarketOrder => {
                let request: MarketOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_market_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            SpotRequestType::LimitOrder => {
                let request: LimitOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_limit_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            SpotRequestType::UpdateOrder => {
                let request: UpdateOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_update_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
            SpotRequestType::CancelOrder => {
                let request: CancelOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                match self
                    .get_orderbook(request.base_asset_id, request.quote_asset_id)?
                    .place_cancel_order(&sender, &request)
                {
                    Ok(_ordering_processing_result) => Ok(()),
                    Err(_err) => Err(GDEXError::OrderRequest),
                }
            }
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

impl EventEmitter for SpotController {
    fn get_event_manager(&mut self) -> &mut Arc<Mutex<EventManager>> {
        &mut self.event_manager
    }
}

impl SpotController {
    // HELPER FUNCTIONS

    pub fn get_orderbook_key(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> AssetPairKey {
        format!("{}_{}", base_asset_id, quote_asset_id)
    }

    pub fn validate_controllerbook_exists(&self, base_asset_id: AssetId, quote_asset_id: AssetId) -> bool {
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

    // METRIC FUNCTIONS

    pub fn get_orderbook(
        &mut self,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
    ) -> Result<&mut SpotOrderbook, GDEXError> {
        let lookup_string = self.get_orderbook_key(base_asset_id, quote_asset_id);
        self.orderbooks.get_mut(&lookup_string).ok_or(GDEXError::AccountLookup)
    }

    // USER FUNCTIONS

    pub fn create_orderbook(&mut self, base_asset_id: AssetId, quote_asset_id: AssetId) -> Result<(), GDEXError> {
        let lookup_string = self.get_orderbook_key(base_asset_id, quote_asset_id);
        if !self.validate_controllerbook_exists(base_asset_id, quote_asset_id) {
            self.orderbooks.insert(
                lookup_string,
                SpotOrderbook::new(
                    base_asset_id,
                    quote_asset_id,
                    self.controller_account.clone(),
                    Arc::clone(&self.bank_controller),
                    Arc::clone(&self.event_manager),
                ),
            );
            Ok(())
        } else {
            Err(GDEXError::OrderBookCreation)
        }
    }
}

// ORDER BOOK INTERFACE

/// Creates a single orderbook instance and verifies all interactions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpotOrderbook {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    controller_account: AccountPubKey,
    bank_controller: Arc<Mutex<BankController>>,
    orderbook: Orderbook,
    order_to_account: HashMap<OrderId, AccountPubKey>,
    // shared
    event_manager: Arc<Mutex<EventManager>>,
}

impl SpotOrderbook {
    pub fn new(
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        controller_account: AccountPubKey,
        bank_controller: Arc<Mutex<BankController>>,
        event_manager: Arc<Mutex<EventManager>>,
    ) -> Self {
        assert!(base_asset_id != quote_asset_id);
        let orderbook = Orderbook::new(base_asset_id, quote_asset_id);
        SpotOrderbook {
            base_asset_id,
            quote_asset_id,
            controller_account,
            bank_controller,
            orderbook,
            order_to_account: HashMap::new(),
            event_manager,
        }
    }

    // METRIC FUNCTIONS

    pub fn get_orderbook_depth(&self) -> OrderbookDepth {
        self.orderbook.get_orderbook_depth()
    }

    // TODO - https://github.com/gdexorg/gdex/issues/172 - Restrict overwrite_orderbook to benchmark only
    pub fn overwrite_orderbook(&mut self, new_orderbook: Orderbook) {
        self.order_to_account = HashMap::new();
        self.orderbook = new_orderbook;
    }

    // HELPER FUNCTIONS

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

    fn send_base_asset(&self, account: &AccountPubKey, quantity: u64) -> Result<(), GDEXError> {
        self.bank_controller.lock().unwrap().transfer(
            &self.controller_account,
            account,
            self.base_asset_id,
            quantity,
        )?;
        Ok(())
    }

    fn send_quote_asset(&self, account: &AccountPubKey, quantity: u64) -> Result<(), GDEXError> {
        self.bank_controller.lock().unwrap().transfer(
            &self.controller_account,
            account,
            self.quote_asset_id,
            quantity,
        )?;
        Ok(())
    }

    fn receive_base_asset(&self, account: &AccountPubKey, quantity: u64) -> Result<(), GDEXError> {
        self.bank_controller.lock().unwrap().transfer(
            account,
            &self.controller_account,
            self.base_asset_id,
            quantity,
        )?;
        Ok(())
    }

    fn receive_quote_asset(&self, account: &AccountPubKey, quantity: u64) -> Result<(), GDEXError> {
        self.bank_controller.lock().unwrap().transfer(
            account,
            &self.controller_account,
            self.quote_asset_id,
            quantity,
        )?;
        Ok(())
    }
}

impl EventEmitter for SpotOrderbook {
    fn get_event_manager(&mut self) -> &mut Arc<Mutex<EventManager>> {
        &mut self.event_manager
    }
}

impl OrderBookWrapper for SpotOrderbook {
    // HELPER FUNCTIONS

    // GETTERS
    fn get_orderbook(&mut self) -> &mut Orderbook {
        &mut self.orderbook
    }

    fn get_pub_key_from_order_id(&self, order_id: &OrderId) -> AccountPubKey {
        self.order_to_account
            .get(order_id)
            .ok_or(GDEXError::AccountLookup)
            .unwrap()
            .clone()
    }

    // SETTERS
    fn set_order(&mut self, order_id: OrderId, account: AccountPubKey) -> Result<(), GDEXError> {
        // // order id should be constantly increasing
        if self.order_to_account.contains_key(&order_id) {
            return Err(GDEXError::OrderRequest);
        }
        self.order_to_account.insert(order_id, account);
        Ok(())
    }

    fn validate_controller(
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

    // USER FUNCTIONS

    fn update_state_on_limit_order_creation(
        &mut self,
        account: &AccountPubKey,
        _order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            self.receive_base_asset(account, quantity)?;
        } else {
            self.receive_quote_asset(account, quantity * price)?;
        }
        Ok(())
    }

    fn update_state_on_fill(
        &mut self,
        account: &AccountPubKey,
        _order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            self.send_quote_asset(account, quantity * price)?;
        } else {
            self.send_base_asset(account, quantity)?;
        }
        Ok(())
    }

    #[allow(clippy::collapsible_else_if)]
    fn update_state_on_update(
        &mut self,
        account: &AccountPubKey,
        _order_id: u64,
        side: OrderSide,
        previous_price: u64,
        previous_quantity: u64,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            if quantity > previous_quantity {
                self.receive_base_asset(account, quantity - previous_quantity)?;
            } else {
                self.send_base_asset(account, previous_quantity - quantity)?;
            }
        } else {
            if quantity * price > previous_quantity * previous_price {
                self.receive_quote_asset(account, quantity * price - previous_quantity * previous_price)?;
            } else {
                self.send_quote_asset(account, previous_quantity * previous_price - quantity * price)?;
            }
        }
        Ok(())
    }

    fn update_state_on_cancel(
        &mut self,
        account: &AccountPubKey,
        _order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if matches!(side, OrderSide::Ask) {
            self.send_base_asset(account, quantity)?;
        } else {
            self.send_quote_asset(account, quantity * price)?;
        }
        Ok(())
    }

    // event emitters

    fn emit_order_new_event(&mut self, account: &AccountPubKey, order_id: u64, side: u64, price: u64, quantity: u64) {
        self.emit_event(&SpotOrderNewEvent::new(account, order_id, side, price, quantity));
    }

    fn emit_order_partial_fill_event(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: u64,
        price: u64,
        quantity: u64,
    ) {
        self.emit_event(&SpotOrderFillEvent::new(account, order_id, side, price, quantity));
    }

    fn emit_order_fill_event(&mut self, account: &AccountPubKey, order_id: u64, side: u64, price: u64, quantity: u64) {
        self.emit_event(&SpotOrderPartialFillEvent::new(
            account, order_id, side, price, quantity,
        ));
    }

    fn emit_order_update_event(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: u64,
        price: u64,
        quantity: u64,
    ) {
        self.emit_event(&SpotOrderUpdateEvent::new(account, order_id, side, price, quantity));
    }

    fn emit_order_cancel_event(&mut self, account: &AccountPubKey, order_id: u64) {
        self.emit_event(&SpotOrderCancelEvent::new(account, order_id));
    }
}

// TESTS

#[cfg(test)]
pub mod spot_tests {
    // crate
    use super::*;
    use crate::bank::controller::{BankController, CREATED_ASSET_BALANCE};

    // gdex
    use gdex_types::crypto::KeypairTraits;
    use gdex_types::{
        account::account_test_functions::generate_keypair_vec,
        block::BlockDigest,
        order_book::{OrderProcessingResult, OrderSide, Success},
    };

    // mysten
    use fastcrypto::DIGEST_LEN;

    // constants

    const BASE_ASSET_ID: AssetId = 0;
    const QUOTE_ASSET_ID: AssetId = 1;
    const TRANSFER_AMOUNT: u64 = 1_000_000;

    // test helpers

    fn place_limit_order_helper(
        orderbook_interface: &mut SpotOrderbook,
        account: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> OrderProcessingResult {
        let limit_order_request = LimitOrderRequest::new(BASE_ASSET_ID, QUOTE_ASSET_ID, side as u64, price, quantity);
        orderbook_interface
            .place_limit_order(account, &limit_order_request)
            .unwrap()
    }

    fn place_update_order_helper(
        orderbook_interface: &mut SpotOrderbook,
        account: &AccountPubKey,
        side: OrderSide,
        price: u64,
        quantity: u64,
        order_id: u64,
    ) -> OrderProcessingResult {
        let update_order_request =
            UpdateOrderRequest::new(BASE_ASSET_ID, QUOTE_ASSET_ID, side as u64, price, quantity, order_id);
        orderbook_interface
            .place_update_order(account, &update_order_request)
            .unwrap()
    }

    fn place_cancel_order_helper(
        orderbook_interface: &mut SpotOrderbook,
        account: &AccountPubKey,
        side: OrderSide,
        order_id: u64,
    ) -> OrderProcessingResult {
        let cancel_order_request = CancelOrderRequest::new(BASE_ASSET_ID, QUOTE_ASSET_ID, side as u64, order_id);
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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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

        let controller_router = ControllerRouter::default();
        controller_router.initialize_controllers();
        controller_router.initialize_controller_accounts();

        controller_router
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(account.public())
            .unwrap();
        controller_router
            .bank_controller
            .lock()
            .unwrap()
            .create_asset(account.public())
            .unwrap();

        controller_router
            .spot_controller
            .lock()
            .unwrap()
            .create_orderbook(BASE_ASSET_ID, QUOTE_ASSET_ID)
            .unwrap();

        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let bid_size = 100;
        let bid_price = 100;
        let transaction = create_limit_order_transaction(
            account.public(),
            recent_block_hash,
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid as u64,
            bid_price,
            bid_size,
        );
        controller_router
            .spot_controller
            .lock()
            .unwrap()
            .handle_consensus_transaction(&transaction)
            .unwrap();

        let bank_controller_ref = Arc::clone(&controller_router.bank_controller);

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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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

        let event_manager = EventManager::new();
        let event_manager_ref = Arc::new(Mutex::new(event_manager));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref.lock().unwrap().create_account(&controller_account);

        let mut orderbook_interface = SpotOrderbook::new(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            controller_account,
            Arc::clone(&bank_controller_ref),
            Arc::clone(&event_manager_ref),
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
