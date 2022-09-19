//! Copyright (c) 2018 Anton Dort-Golts
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//!
//! The orderbook holds functions responsible for running an orderbook application
//!
//! note, orderbook has commented out line 390 to avoid random failures when submitting transasctions in quick succession
//! this uniqueness check in the orderbook is seems potentially incorrect, or strange, as it includes the timestamp of the order
//! we should include some sort of random noise to ensure that every order that touches the book gets inserted
//! as upstream checks will robustly ensure no duplicates

use super::order_queues::OrderQueue;
use super::orders::{create_cancel_order_request, create_limit_order_request, create_update_order_request};
use super::sequence;
use super::validation::OrderRequestValidator;

use gdex_types::{
    account::AccountPubKey,
    asset::AssetId,
    error::GDEXError,
    order_book::{
        Depth, Failed, Order, OrderProcessingResult, OrderRequest, OrderSide, OrderType, OrderbookDepth, Success,
    },
    transaction::{parse_order_side, CancelOrderRequest, LimitOrderRequest, MarketOrderRequest, UpdateOrderRequest},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

pub type OrderId = u64;

const MIN_SEQUENCE_ID: u64 = 1;
const MAX_SEQUENCE_ID: u64 = 1_000_000;
const MAX_STALLED_INDICES_IN_QUEUE: u64 = 10;
const ORDER_QUEUE_INIT_CAPACITY: usize = 500;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Orderbook {
    base_asset: AssetId,
    quote_asset: AssetId,
    bid_queue: OrderQueue<Order>,
    ask_queue: OrderQueue<Order>,
    seq: sequence::TradeSequence,
    order_validator: OrderRequestValidator,
}

impl Orderbook {
    /// Create new orderbook for pair of assets
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    ///// let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
    ///// let result = orderbook.process_order(OrderRequest::MarketOrder{  });
    ///// assert_eq!(orderbook)
    /// ```
    // todo fix doc test!
    pub fn new(base_asset: AssetId, quote_asset: AssetId) -> Self {
        Orderbook {
            base_asset,
            quote_asset,
            bid_queue: OrderQueue::new(OrderSide::Bid, MAX_STALLED_INDICES_IN_QUEUE, ORDER_QUEUE_INIT_CAPACITY),
            ask_queue: OrderQueue::new(OrderSide::Ask, MAX_STALLED_INDICES_IN_QUEUE, ORDER_QUEUE_INIT_CAPACITY),
            seq: sequence::new_sequence_gen(MIN_SEQUENCE_ID, MAX_SEQUENCE_ID),
            order_validator: OrderRequestValidator::new(base_asset, quote_asset, MIN_SEQUENCE_ID, MAX_SEQUENCE_ID),
        }
    }

    pub fn get_orderbook_depth(&self) -> OrderbookDepth {
        let mut bids_map: HashMap<u64, u64> = HashMap::new();
        let mut asks_map: HashMap<u64, u64> = HashMap::new();

        // aggregate bids + asks
        for order in self.bid_queue.orders.values() {
            let price = order.get_price();
            match bids_map.entry(price) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    e.insert(e.get() + order.get_quantity());
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(order.get_quantity());
                }
            }
        }
        for order in self.ask_queue.orders.values() {
            let price = order.get_price();
            match asks_map.entry(price) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    e.insert(e.get() + order.get_quantity());
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(order.get_quantity());
                }
            }
        }

        let mut bids: Vec<Depth> = Vec::new();
        let mut asks: Vec<Depth> = Vec::new();

        for (price, quantity) in &bids_map {
            bids.push(Depth {
                price: *price,
                quantity: *quantity,
            });
        }
        for (price, quantity) in &asks_map {
            asks.push(Depth {
                price: *price,
                quantity: *quantity,
            });
        }

        bids.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        OrderbookDepth { bids, asks }
    }

    pub fn process_order(&mut self, order: OrderRequest) -> OrderProcessingResult {
        // processing result accumulator
        let mut process_result: OrderProcessingResult = vec![];

        // validate request
        if let Err(reason) = self.order_validator.validate(&order) {
            process_result.push(Err(Failed::Validation(String::from(reason))));
            return process_result;
        }

        match order {
            OrderRequest::Market {
                base_asset_id,
                quote_asset_id,
                side,
                quantity,
                ..
            } => {
                // generate new ID for order
                let order_id = self.seq.next_id();
                let price: u64 = 0;
                process_result.push(Ok(Success::Accepted {
                    order_id,
                    side,
                    price,
                    quantity,
                    order_type: OrderType::Market,
                    timestamp: SystemTime::now(),
                }));

                self.process_market_order(
                    &mut process_result,
                    order_id,
                    base_asset_id,
                    quote_asset_id,
                    side,
                    quantity,
                );
            }

            OrderRequest::Limit {
                base_asset_id,
                quote_asset_id,
                side,
                price,
                quantity,
                local_timestamp,
            } => {
                let order_id = self.seq.next_id();
                process_result.push(Ok(Success::Accepted {
                    order_id,
                    side,
                    price,
                    quantity,
                    order_type: OrderType::Limit,
                    timestamp: SystemTime::now(),
                }));

                self.process_limit_order(
                    &mut process_result,
                    order_id,
                    base_asset_id,
                    quote_asset_id,
                    side,
                    price,
                    quantity,
                    local_timestamp,
                );
            }

            OrderRequest::Update {
                order_id,
                side,
                price,
                quantity,
                local_timestamp,
                ..
            } => {
                self.process_order_update(&mut process_result, order_id, side, price, quantity, local_timestamp);
            }

            OrderRequest::Cancel { order_id, side, .. } => {
                self.process_order_cancel(&mut process_result, order_id, side);
            }
        }

        // return collected processing results

        process_result
    }

    /// Get current spread as a tuple: (bid, ask)
    pub fn current_spread(&mut self) -> Option<(u64, u64)> {
        let bid = self.bid_queue.peek()?.price;
        let ask = self.ask_queue.peek()?.price;
        Some((bid, ask))
    }

    /* Processing logic */

    fn process_market_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        quantity: u64,
    ) {
        // get copy of the current limit order
        let opposite_order_result = {
            let opposite_queue = match side {
                OrderSide::Bid => &mut self.ask_queue,
                OrderSide::Ask => &mut self.bid_queue,
            };
            opposite_queue.peek().cloned()
        };

        if let Some(opposite_order) = opposite_order_result {
            let matching_complete = self.order_matching(
                results,
                &opposite_order,
                order_id,
                base_asset,
                quote_asset,
                OrderType::Market,
                side,
                quantity,
            );

            if !matching_complete {
                // match the rest
                self.process_market_order(
                    results,
                    order_id,
                    base_asset,
                    quote_asset,
                    side,
                    quantity - opposite_order.quantity,
                );
            }
        } else {
            // no limit orders found
            results.push(Err(Failed::NoMatch(order_id)));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    ) {
        // take a look at current opposite limit order
        let opposite_order_result = {
            let opposite_queue = match side {
                OrderSide::Bid => &mut self.ask_queue,
                OrderSide::Ask => &mut self.bid_queue,
            };
            opposite_queue.peek().cloned()
        };

        if let Some(opposite_order) = opposite_order_result {
            let could_be_matched = match side {
                // verify bid/ask price overlap
                OrderSide::Bid => price >= opposite_order.price,
                OrderSide::Ask => price <= opposite_order.price,
            };

            if could_be_matched {
                // match immediately
                let matching_complete = self.order_matching(
                    results,
                    &opposite_order,
                    order_id,
                    base_asset,
                    quote_asset,
                    OrderType::Limit,
                    side,
                    quantity,
                );

                if !matching_complete {
                    // process the rest of new limit order
                    self.process_limit_order(
                        results,
                        order_id,
                        base_asset,
                        quote_asset,
                        side,
                        price,
                        quantity - opposite_order.quantity,
                        timestamp,
                    );
                }
            } else {
                // just insert new order in queue
                self.store_new_limit_order(
                    results,
                    order_id,
                    base_asset,
                    quote_asset,
                    side,
                    price,
                    quantity,
                    timestamp,
                );
            }
        } else {
            self.store_new_limit_order(
                results,
                order_id,
                base_asset,
                quote_asset,
                side,
                price,
                quantity,
                timestamp,
            );
        }
    }

    fn process_order_update(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        let current_order = order_queue.get_order(order_id).unwrap();
        let previous_quantity = current_order.get_quantity();
        let previous_price = current_order.get_price();

        if order_queue.update(
            order_id,
            price,
            timestamp,
            Order {
                order_id,
                base_asset: self.base_asset,
                quote_asset: self.quote_asset,
                side,
                price,
                quantity,
            },
        ) {
            results.push(Ok(Success::Updated {
                order_id,
                side,
                previous_quantity,
                previous_price,
                price,
                quantity,
                timestamp: SystemTime::now(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }

    fn process_order_cancel(&mut self, results: &mut OrderProcessingResult, order_id: u64, side: OrderSide) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        // get order to extract price + quantity
        if let Some(order) = order_queue.get_order(order_id) {
            // get price and quantity of live order
            let price = order.get_price();
            let quantity = order.get_quantity();

            if order_queue.cancel(order_id) {
                results.push(Ok(Success::Cancelled {
                    order_id,
                    side,
                    price,
                    quantity,
                    timestamp: SystemTime::now(),
                }));
            }
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }

    /* Helpers */

    pub fn get_order(&mut self, side: OrderSide, order_id: u64) -> Result<&Order, Failed> {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        order_queue.get_order(order_id).ok_or(Failed::OrderNotFound(order_id))
    }

    #[allow(clippy::too_many_arguments)]
    fn store_new_limit_order(
        &mut self,
        _results: &mut OrderProcessingResult,
        order_id: u64,
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };
        if !order_queue.insert(
            order_id,
            price,
            timestamp,
            Order {
                order_id,
                base_asset,
                quote_asset,
                side,
                price,
                quantity,
            },
        ) {
            // results.push(Err(Failed::DuplicateOrderID(order_id)))
        };
    }

    #[allow(clippy::too_many_arguments)]
    fn order_matching(
        &mut self,
        results: &mut OrderProcessingResult,
        opposite_order: &Order,
        order_id: u64,
        base_asset: AssetId,
        quote_asset: AssetId,
        order_type: OrderType,
        side: OrderSide,
        quantity: u64,
    ) -> bool {
        // real processing time
        let deal_time = SystemTime::now();
        // match immediately
        match quantity {
            x if x < opposite_order.quantity => {
                // fill new limit and modify opposite limit

                // report filled new order
                results.push(Ok(Success::Filled {
                    order_id,
                    side,
                    order_type,
                    price: opposite_order.price,
                    quantity,
                    timestamp: deal_time,
                }));

                // report partially filled opposite limit order
                results.push(Ok(Success::PartiallyFilled {
                    order_id: opposite_order.order_id,
                    side: opposite_order.side,
                    order_type: OrderType::Limit,
                    price: opposite_order.price,
                    quantity,
                    timestamp: deal_time,
                }));

                // modify unmatched part of the opposite limit order
                {
                    let opposite_queue = match side {
                        OrderSide::Bid => &mut self.ask_queue,
                        OrderSide::Ask => &mut self.bid_queue,
                    };
                    opposite_queue.modify_current_order(Order {
                        order_id: opposite_order.order_id,
                        base_asset,
                        quote_asset,
                        side: opposite_order.side,
                        price: opposite_order.price,
                        quantity: opposite_order.quantity - quantity,
                    });
                }
            }
            x if x > opposite_order.quantity => {
                // partially fill new limit order, fill opposite limit and notify to process the rest
                // report new order partially filled
                results.push(Ok(Success::PartiallyFilled {
                    order_id,
                    side,
                    order_type,
                    price: opposite_order.price,
                    quantity: opposite_order.quantity,
                    timestamp: deal_time,
                }));

                // report filled opposite limit order
                results.push(Ok(Success::Filled {
                    order_id: opposite_order.order_id,
                    side: opposite_order.side,
                    order_type: OrderType::Limit,
                    price: opposite_order.price,
                    quantity: opposite_order.quantity,
                    timestamp: deal_time,
                }));

                // remove filled limit order from the queue
                {
                    let opposite_queue = match side {
                        OrderSide::Bid => &mut self.ask_queue,
                        OrderSide::Ask => &mut self.bid_queue,
                    };
                    opposite_queue.pop();
                }

                // matching incomplete
                return false;
            }
            _ => {
                // orders exactly match -> fill both and remove old limit
                // report filled new order
                results.push(Ok(Success::Filled {
                    order_id,
                    side,
                    order_type,
                    price: opposite_order.price,
                    quantity,
                    timestamp: deal_time,
                }));
                // report filled opposite limit order
                results.push(Ok(Success::Filled {
                    order_id: opposite_order.order_id,
                    side: opposite_order.side,
                    order_type: OrderType::Limit,
                    price: opposite_order.price,
                    quantity,
                    timestamp: deal_time,
                }));

                // remove filled limit order from the queue
                {
                    let opposite_queue = match side {
                        OrderSide::Bid => &mut self.ask_queue,
                        OrderSide::Ask => &mut self.bid_queue,
                    };
                    opposite_queue.pop();
                }
            }
        }
        // complete matching
        true
    }
}

pub trait OrderBookWrapper {
    // HELPER FUNCTIONS

    // GETTERS
    fn get_orderbook(&mut self) -> &mut Orderbook;

    fn get_pub_key_from_order_id(&self, order_id: &OrderId) -> AccountPubKey;

    // SETTERS
    fn set_order(&mut self, order_id: OrderId, account: AccountPubKey) -> Result<(), GDEXError>;

    // TODO - remove gating from the order_book level
    // this creates awkward tension in any instance of cross-margin
    fn validate_controller(
        &self,
        account: &AccountPubKey,
        side: OrderSide,
        quantity: u64,
        price: u64,
        previous_quantity: u64,
        previous_price: u64,
    ) -> Result<(), GDEXError>;

    // PLACERS [ORDERS]

    // PLACE MARKET ORDER : TODO : UNIMPLEMENTED

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
        // parse side
        let side = parse_order_side(request.side)?;

        // check balances before placing order
        self.validate_controller(account, side, request.quantity, request.price, 0, 0)?;

        // create and process limit order
        let order = create_limit_order_request(
            request.base_asset_id,
            request.quote_asset_id,
            side,
            request.price,
            request.quantity,
            SystemTime::now(),
        );
        let res = self.get_orderbook().process_order(order);
        self.process_order_result(account, res)
    }

    // PLACE CANCEL ORDER

    fn place_cancel_order(
        &mut self,
        account: &AccountPubKey,
        request: &CancelOrderRequest,
    ) -> Result<OrderProcessingResult, GDEXError> {
        let side = parse_order_side(request.side)?;

        // create and process limit order
        let order = create_cancel_order_request(
            request.base_asset_id,
            request.quote_asset_id,
            request.order_id,
            side,
            SystemTime::now(),
        );
        let res = self.get_orderbook().process_order(order);
        self.process_order_result(account, res)
    }

    // PLACE UPDATE ORDER

    fn place_update_order(
        &mut self,
        account: &AccountPubKey,
        request: &UpdateOrderRequest,
    ) -> Result<OrderProcessingResult, GDEXError> {
        // parse side
        let side = parse_order_side(request.side)?;

        // check updates against user's balances
        let current_order = self.get_orderbook().get_order(side, request.order_id).unwrap();
        let current_quantity = current_order.get_quantity();
        let current_price = current_order.get_price();

        // check balances before placing order
        self.validate_controller(
            account,
            side,
            request.quantity - current_quantity,
            request.price,
            current_quantity,
            current_price,
        )?;

        // create and process limit order
        let order = create_update_order_request(
            request.base_asset_id,
            request.quote_asset_id,
            request.order_id,
            side,
            request.price,
            request.quantity,
            SystemTime::now(),
        );
        let res = self.get_orderbook().process_order(order);
        self.process_order_result(account, res)
    }

    // helper functions for process_order_result

    fn update_state_on_limit_order_creation(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError>;

    fn update_state_on_fill(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError>;

    #[allow(clippy::too_many_arguments)]
    fn update_state_on_update(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: OrderSide,
        previous_price: u64,
        previous_quantity: u64,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError>;

    fn update_state_on_cancel(
        &mut self,
        account: &AccountPubKey,
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) -> Result<(), GDEXError>;

    fn process_order_result(
        &mut self,
        account: &AccountPubKey,
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
                        self.update_state_on_limit_order_creation(account, *order_id, *side, *price, *quantity)?;
                    }
                    // insert new order to map
                    self.set_order(*order_id, account.clone())?;
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
                    let existing_pub_key = self.get_pub_key_from_order_id(order_id);
                    self.update_state_on_fill(&existing_pub_key, *order_id, *side, *price, *quantity)?;
                }
                Ok(Success::Filled {
                    order_id,
                    side,
                    price,
                    quantity,
                    ..
                }) => {
                    // update user balances
                    let existing_pub_key = self.get_pub_key_from_order_id(order_id);
                    self.update_state_on_fill(&existing_pub_key, *order_id, *side, *price, *quantity)?;
                    // TODO - Uncomment remove below after diagnosing how this can cause failures
                    // remove order from map
                    //self.order_to_account.remove(order_id).ok_or(GDEXError::OrderRequest)?;
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
                    let existing_pub_key = self.get_pub_key_from_order_id(order_id);
                    self.update_state_on_update(
                        &existing_pub_key,
                        *order_id,
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
                    let existing_pub_key = self.get_pub_key_from_order_id(order_id);
                    self.update_state_on_cancel(&existing_pub_key, *order_id, *side, *price, *quantity)?;
                }
                Err(_failure) => {
                    return Err(GDEXError::OrderRequest);
                }
            }
        }
        Ok(res)
    }
}

#[cfg(test)]
mod test_order_book {

    use super::*;
    use crate::orders::{
        create_cancel_order_request, create_limit_order_request, create_market_order_request,
        create_update_order_request,
    };

    const BASE_ASSET: u64 = 0;
    const QUOTE_ASSET: u64 = 1;

    #[test]
    fn failed_cancel() {
        let mut orderbook = Orderbook::new(BASE_ASSET, QUOTE_ASSET);
        let request = create_cancel_order_request(BASE_ASSET, QUOTE_ASSET, 1, OrderSide::Bid, SystemTime::now());
        let mut result = orderbook.process_order(request);

        assert_eq!(result.len(), 1);
        match result.pop().unwrap() {
            Err(..) => (),
            _ => panic!("unexpected success"),
        }
    }

    #[should_panic]
    #[test]
    pub fn failed_match() {
        let mut order_book = Orderbook::new(BASE_ASSET, QUOTE_ASSET);

        // create and process limit order
        let order = create_limit_order_request(BASE_ASSET, QUOTE_ASSET + 1, OrderSide::Ask, 10, 1, SystemTime::now());
        let results = order_book.process_order(order);
        for result in results {
            result.unwrap();
        }
    }

    #[test]
    pub fn successful_match() {
        let mut order_book = Orderbook::new(BASE_ASSET, QUOTE_ASSET);

        // create and process limit order
        let order = create_limit_order_request(BASE_ASSET, QUOTE_ASSET, OrderSide::Bid, 10, 100, SystemTime::now());
        order_book.process_order(order);

        // create and process limit order
        let order = create_market_order_request(BASE_ASSET, QUOTE_ASSET, OrderSide::Ask, 10, SystemTime::now());
        let results = order_book.process_order(order);
        for result in results {
            result.unwrap();
        }
    }

    #[test]
    pub fn successful_update() {
        let mut order_book = Orderbook::new(BASE_ASSET, QUOTE_ASSET);

        // create and process limit order
        let order = create_limit_order_request(BASE_ASSET, QUOTE_ASSET, OrderSide::Bid, 10, 100, SystemTime::now());
        let mut results = order_book.process_order(order);

        let order_result = results.pop().unwrap().unwrap();

        match order_result {
            Success::Accepted { order_id, .. } => {
                let update_order = create_update_order_request(
                    BASE_ASSET,
                    QUOTE_ASSET,
                    order_id,
                    OrderSide::Bid,
                    100,
                    100,
                    SystemTime::now(),
                );
                order_book.process_order(update_order).pop().unwrap().unwrap();
            }
            _ => {
                panic!("unexpected match result");
            }
        }
    }

    #[test]
    pub fn partial_match_limits() {
        let mut order_book = Orderbook::new(BASE_ASSET, QUOTE_ASSET);

        // create and process limit order
        let order = create_limit_order_request(BASE_ASSET, QUOTE_ASSET, OrderSide::Bid, 10, 100, SystemTime::now());
        order_book.process_order(order);

        let order = create_limit_order_request(BASE_ASSET, QUOTE_ASSET, OrderSide::Ask, 5, 10, SystemTime::now());
        let results = order_book.process_order(order);

        for result in results {
            result.unwrap();
        }
    }
}
