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
use super::sequence;
use super::validation::OrderRequestValidator;
use gdex_types::{
    asset::AssetId,
    order_book::{Failed, Order, OrderProcessingResult, OrderSide, OrderType, Success},
    transaction::OrderRequest,
};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

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

            OrderRequest::CancelOrder { order_id, side, .. } => {
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
