// crate
use crate::bank::controller::BankController;
use crate::controller::Controller;
use crate::futures::{proto::*, types::*, utils::*};
use crate::router::ControllerRouter;
use crate::spot::proto::*;
use crate::utils::engine::order_book::{OrderBookWrapper, OrderId, Orderbook};
// TODO - include continuous OI calculation for FuturesMarket

// gdex
use gdex_types::{
    account::AccountPubKey,
    crypto::ToFromBytes,
    error::GDEXError,
    order_book::OrderSide,
    store::ProcessBlockStore,
    transaction::{deserialize_protobuf, FuturesOrder, FuturesPosition, Transaction},
};
// external
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryInto,
    sync::{Arc, Mutex},
};

// CONSTANTS

pub const FUTURES_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"FUTURESSSCONTROLLERAAAAAAAAAAAAA";
const DEFAULT_MAX_LEVERAGE: u64 = 20;

// INTERFACE

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuturesController {
    pub controller_account: AccountPubKey,
    bank_controller: Arc<Mutex<BankController>>,
    // A market_place is created by an admin
    // and is a collection of futures market interfaces
    market_places: HashMap<AccountPubKey, Marketplace>,
}

impl Default for FuturesController {
    fn default() -> Self {
        Self {
            controller_account: AccountPubKey::from_bytes(FUTURES_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            bank_controller: Arc::new(Mutex::new(BankController::default())), // TEMPORARY
            market_places: HashMap::new(),
        }
    }
}

// STRUCT IMPLS

impl FuturesController {
    pub fn new(controller_account: AccountPubKey, bank_controller: Arc<Mutex<BankController>>) -> Self {
        Self {
            controller_account,
            bank_controller,
            market_places: HashMap::new(),
        }
    }

    fn create_marketplace(
        &mut self,
        market_admin: AccountPubKey,
        request: CreateMarketplaceRequest,
    ) -> Result<(), GDEXError> {
        // ensure that market does not already exist
        if self.market_places.get(&market_admin).is_some() {
            return Err(GDEXError::MarketplaceExistence);
        }

        // ensure that market_admin is being initialized to a new account
        if market_admin == self.controller_account {
            return Err(GDEXError::FuturesInitialization);
        }

        // TODO - check that quote asset exists
        // TODO - add rails against arbitrary accounts creating markets
        self.market_places.insert(
            market_admin,
            Marketplace {
                deposits: Arc::new(Mutex::new(HashMap::new())),
                quote_asset_id: request.quote_asset_id,
                latest_time: 0,
                markets: HashMap::new(),
            },
        );
        Ok(())
    }

    fn create_market(&mut self, market_admin: AccountPubKey, request: CreateMarketRequest) -> Result<(), GDEXError> {
        // TODO - Check that quote asset does not match base asset
        // ensure that the market place is valid
        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            // if the market has already been created, return an error
            if market_place.markets.get(&request.base_asset_id).is_some() {
                return Err(GDEXError::MarketExistence);
            }
            market_place.markets.insert(
                request.base_asset_id,
                FuturesMarket {
                    open_interest: 0,
                    last_traded_price: 0,
                    oracle_price: 0,
                    max_leverage: DEFAULT_MAX_LEVERAGE,
                    base_asset_id: request.base_asset_id,
                    quote_asset_id: market_place.quote_asset_id,
                    accounts: HashMap::new(),
                    order_to_account: HashMap::new(),
                    orderbook: Orderbook::new(request.base_asset_id, market_place.quote_asset_id),
                    marketplace_deposits: Arc::downgrade(&market_place.deposits),
                },
            );
        } else {
            return Err(GDEXError::MarketplaceExistence);
        }
        Ok(())
    }

    fn update_market_params(
        &mut self,
        market_admin: AccountPubKey,
        request: UpdateMarketParamsRequest,
    ) -> Result<(), GDEXError> {
        // TODO - Check that quote asset does not match base asset
        // ensure that the market place is valid
        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            if let Some(market) = market_place.markets.get_mut(&request.base_asset_id) {
                // max leverage cannot be decreased
                if market.max_leverage > request.max_leverage {
                    return Err(GDEXError::FuturesUpdate);
                }
                market.max_leverage = request.max_leverage;
            } else {
                return Err(GDEXError::MarketExistence);
            }
        } else {
            return Err(GDEXError::MarketplaceExistence);
        }
        Ok(())
    }

    fn update_time(&mut self, market_admin: AccountPubKey, request: UpdateTimeRequest) -> Result<(), GDEXError> {
        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            market_place.latest_time = request.latest_time;
        } else {
            return Err(GDEXError::MarketplaceExistence);
        };
        Ok(())
    }

    fn update_prices(&mut self, market_admin: AccountPubKey, request: UpdatePricesRequest) -> Result<(), GDEXError> {
        // TODO - move to more robust system to ensure that the prices are being updated in the correct order
        if request.latest_prices.len() != self.market_places.len() {
            return Err(GDEXError::MarketPrices);
        }

        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            for (counter, (_asset_id, market)) in market_place.markets.iter_mut().enumerate() {
                market.oracle_price = request.latest_prices[counter];
            }
        } else {
            return Err(GDEXError::MarketplaceExistence);
        };
        Ok(())
    }

    fn account_deposit(&mut self, sender: AccountPubKey, request: AccountDepositRequest) -> Result<(), GDEXError> {
        let market_admin = AccountPubKey::from_bytes(&request.market_admin).map_err(|_| GDEXError::InvalidAddress)?;

        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            let mut bank_controller = self.bank_controller.lock().unwrap();
            // initialize the account for the receiver by sending a payment transaction
            bank_controller.transfer(
                &sender,
                &self.controller_account,
                market_place.quote_asset_id,
                request.quantity.try_into().map_err(|_| GDEXError::Conversion)?,
            )?;

            let mut deposit_lock = market_place.deposits.lock().unwrap();
            // check if deposits contains sender, if not create the account and fund
            if let Some(deposit) = deposit_lock.get_mut(&sender) {
                *deposit = request.quantity;
            } else {
                deposit_lock.insert(sender, request.quantity);
            }
        } else {
            return Err(GDEXError::MarketplaceExistence);
        };
        Ok(())
    }

    fn account_withdraw(&mut self, sender: AccountPubKey, request: AccountWithdrawalRequest) -> Result<(), GDEXError> {
        let market_admin = AccountPubKey::from_bytes(&request.market_admin).map_err(|_| GDEXError::InvalidAddress)?;
        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            let sender_used_collateral: i64 = get_account_total_req_collateral(market_place, &sender, None)?
                .try_into()
                .map_err(|_| GDEXError::Conversion)?;
            let sender_unrealized_pnl = get_account_unrealized_pnl(market_place, &sender)?;

            let mut deposit_lock = market_place.deposits.lock().unwrap();
            let sender_deposit = deposit_lock.get_mut(&sender).ok_or(GDEXError::AccountLookup)?;
            let converted_quantity = request.quantity.try_into().map_err(|_| GDEXError::Conversion)?;
            if (*sender_deposit + sender_unrealized_pnl - sender_used_collateral) < converted_quantity {
                return Err(GDEXError::FuturesWithdrawal);
            }

            let mut bank_controller = self.bank_controller.lock().unwrap();
            bank_controller.transfer(
                &self.controller_account,
                &sender,
                market_place.quote_asset_id,
                request.quantity,
            )?;

            let converted_quantity: i64 = converted_quantity;
            *sender_deposit -= converted_quantity;
        } else {
            return Err(GDEXError::MarketplaceExistence);
        };
        Ok(())
    }

    fn futures_limit_order(
        &mut self,
        sender: AccountPubKey,
        market_admin: AccountPubKey,
        request: FuturesLimitOrderRequest,
    ) -> Result<(), GDEXError> {
        if let Some(market_place) = self.market_places.get_mut(&market_admin) {
            // TODO - consider max orders per account, or some form of min balance increment per order
            let request_collateral_data = Some(CondensedOrder {
                price: request.price,
                side: request.side,
                quantity: request.quantity,
                base_asset_id: request.base_asset_id,
            });
            let sender_req_collateral =
                get_account_total_req_collateral(market_place, &sender, request_collateral_data)?
                    .try_into()
                    .map_err(|_| GDEXError::Conversion)?;
            let sender_unrealized_pnl = get_account_unrealized_pnl(market_place, &sender)?;

            let sender_deposit = *market_place
                .deposits
                .lock()
                .unwrap()
                .get(&sender)
                .ok_or(GDEXError::AccountLookup)?;

            if sender_deposit + sender_unrealized_pnl < sender_req_collateral {
                return Err(GDEXError::InsufficientCollateral);
            }

            let market = market_place
                .markets
                .get_mut(&request.base_asset_id)
                .ok_or(GDEXError::MarketExistence)?;

            market.place_limit_order(&sender, &LimitOrderRequest::from(request))?;
        } else {
            return Err(GDEXError::MarketplaceExistence);
        };
        Ok(())
    }

    pub fn get_marketplace_state(&self, market_admin: &AccountPubKey) -> Result<MarketplaceState, GDEXError> {
        get_marketplace_state(
            self.market_places
                .get(market_admin)
                .ok_or(GDEXError::MarketplaceExistence)?,
        )
    }

    pub fn get_account_state_by_market(
        &self,
        market_admin: &AccountPubKey,
        account: &AccountPubKey,
    ) -> Result<AccountStateByMarket, GDEXError> {
        get_account_state_by_market(
            self.market_places
                .get(market_admin)
                .ok_or(GDEXError::MarketplaceExistence)?,
            account,
        )
    }

    pub fn get_account_total_req_collateral(
        &self,
        market_admin: &AccountPubKey,
        account: &AccountPubKey,
    ) -> Result<u64, GDEXError> {
        get_account_total_req_collateral(
            self.market_places
                .get(market_admin)
                .ok_or(GDEXError::MarketplaceExistence)?,
            account,
            None,
        )
    }

    pub fn get_account_unrealized_pnl(
        &self,
        market_admin: &AccountPubKey,
        account: &AccountPubKey,
    ) -> Result<i64, GDEXError> {
        get_account_unrealized_pnl(
            self.market_places
                .get(market_admin)
                .ok_or(GDEXError::MarketplaceExistence)?,
            account,
        )
    }

    pub fn get_account_available_deposit(
        &self,
        market_admin: &AccountPubKey,
        account: &AccountPubKey,
    ) -> Result<i64, GDEXError> {
        let deposit = *(self
            .market_places
            .get(market_admin)
            .ok_or(GDEXError::MarketplaceExistence)?
            .deposits
            .lock()
            .unwrap()
            .get(account)
            .ok_or(GDEXError::AccountLookup)?);

        let req_collateral: i64 = self
            .get_account_total_req_collateral(market_admin, account)?
            .try_into()
            .map_err(|_| GDEXError::Conversion)?;
        Ok(deposit - req_collateral)
    }
}

#[async_trait]
impl Controller for FuturesController {
    fn initialize(&mut self, controller_router: &ControllerRouter) {
        self.bank_controller = Arc::clone(&controller_router.bank_controller);
    }

    fn initialize_controller_account(&mut self) -> Result<(), GDEXError> {
        // TODO - add initialization after finding appropriate address for controller account
        self.bank_controller
            .lock()
            .unwrap()
            .create_account(&self.controller_account)?;
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError> {
        let sender = transaction.get_sender()?;
        let request_type: FuturesRequestType = transaction.get_request_type()?;
        match request_type {
            FuturesRequestType::CreateMarketplace => {
                let request: CreateMarketplaceRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.create_marketplace(sender, request)?;
            }
            FuturesRequestType::CreateMarket => {
                let request: CreateMarketRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.create_market(sender, request)?;
            }
            FuturesRequestType::UpdateMarketParams => {
                // TODO - add market_admin verification
                let request: UpdateMarketParamsRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.update_market_params(sender, request)?;
            }
            FuturesRequestType::UpdateTime => {
                // TODO - add market_admin verification
                let request: UpdateTimeRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.update_time(sender, request)?;
            }
            FuturesRequestType::UpdatePrices => {
                // TODO - add market_admin verification
                let request: UpdatePricesRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.update_prices(sender, request)?;
            }
            FuturesRequestType::AccountDeposit => {
                let request: AccountDepositRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.account_deposit(sender, request)?;
            }
            FuturesRequestType::AccountWithdrawal => {
                let request: AccountWithdrawalRequest = deserialize_protobuf(&transaction.request_bytes)?;
                self.account_withdraw(sender, request)?;
            }
            FuturesRequestType::FuturesLimitOrder => {
                // TODO - add signature verification
                let request: FuturesLimitOrderRequest = deserialize_protobuf(&transaction.request_bytes)?;
                let market_admin =
                    AccountPubKey::from_bytes(&request.market_admin).map_err(|_| GDEXError::InvalidAddress)?;
                self.futures_limit_order(sender, market_admin, request)?;
            }
        }
        Ok(())
    }

    async fn process_end_of_block(
        _controller: Arc<Mutex<Self>>,
        _process_block_store: &ProcessBlockStore,
        _block_number: u64,
    ) {
    }
}

impl OrderBookWrapper for FuturesMarket {
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
        // order id should be constantly increasing
        if self.order_to_account.contains_key(&order_id) {
            return Err(GDEXError::OrderRequest);
        }
        {
            self.order_to_account.insert(order_id, account);
            Ok(())
        }
    }

    // order check is done upstream because cross-margin calulations are needed
    // doing it here would be require a circular reference to be made between
    // FuturesMarket and the Marketplace
    fn validate_controller(
        &self,
        _account: &AccountPubKey,
        _side: OrderSide,
        _quantity: u64,
        _price: u64,
        _previous_quantity: u64,
        _previous_price: u64,
    ) -> Result<(), GDEXError> {
        Ok(())
    }

    // account FUNCTIONS

    fn update_state_on_limit_order_creation(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        // check if accounts contains account and if not create it
        if !self.accounts.contains_key(account) {
            self.accounts.insert(account.clone(), FuturesAccount::default());
        }
        self.accounts
            .get_mut(account)
            .ok_or(GDEXError::AccountLookup)?
            .open_orders
            .push(FuturesOrder {
                order_id,
                side: side as u64,
                price,
                quantity,
            });
        Ok(())
    }

    fn update_state_on_fill(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        if !self.accounts.contains_key(account) {
            self.accounts.insert(account.clone(), FuturesAccount::default());
        }

        // update last traded price
        self.last_traded_price = price;

        let mut futures_account = self.accounts.get_mut(account).unwrap();
        let marketplace_deposits = self.marketplace_deposits.upgrade().unwrap();
        let mut deposits_lock = marketplace_deposits.lock().unwrap();

        let account_deposit = deposits_lock.get_mut(account).ok_or(GDEXError::AccountLookup)?;

        let new_position = FuturesPosition {
            side: side as u64,
            quantity,
            average_price: price,
        };

        if let Some(old_position) = &futures_account.position {
            let resultant_position = combine_positions(old_position.clone(), new_position.clone());
            if resultant_position.is_some() && resultant_position.as_ref().unwrap().quantity > old_position.quantity {
                // when increasing position, add 1/2 to open interest (1/2 since it is summed for both users)
                self.open_interest += new_position.quantity / 2;
            } else {
                self.open_interest -= new_position.quantity / 2;
            }
            *account_deposit += compute_realized_pnl(old_position, &resultant_position, price)?;
            futures_account.position = resultant_position;
        } else {
            self.open_interest += new_position.quantity;
            futures_account.position = Some(new_position);
        }

        // update open orders
        for (counter, order) in futures_account.open_orders.iter_mut().enumerate() {
            if order.order_id == order_id {
                order.quantity -= quantity;
                if order.quantity == 0 {
                    futures_account.open_orders.remove(counter);
                }
                break;
            }
        }
        Ok(())
    }

    #[allow(clippy::collapsible_else_if)]
    fn update_state_on_update(
        &mut self,
        _account: &AccountPubKey,
        _order_id: u64,
        _side: OrderSide,
        _previous_price: u64,
        _previous_quantity: u64,
        _price: u64,
        _quantity: u64,
    ) -> Result<(), GDEXError> {
        // TODO - implement update
        Err(GDEXError::InvalidRequestTypeError)
    }

    fn update_state_on_cancel(
        &mut self,
        _account: &AccountPubKey,
        _order_id: u64,
        _side: OrderSide,
        _price: u64,
        _quantity: u64,
    ) -> Result<(), GDEXError> {
        // TODO - implement cancel
        Err(GDEXError::InvalidRequestTypeError)
    }
}
