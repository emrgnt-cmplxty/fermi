extern crate rocksdb;

use std::any::Any;
use std::convert::TryInto;
use std::time::SystemTime;
use std::fmt::Debug;
use rocksdb::{ColumnFamilyDescriptor, DB, DBWithThreadMode, Options, SingleThreaded};

use super::domain::{Order, OrderSide, OrderType};
use super::orders::OrderRequest;
use super::order_queues::OrderQueue;
use super::sequence;
use super::validation::OrderRequestValidator;


const MIN_SEQUENCE_ID: u64 = 1;
const MAX_SEQUENCE_ID: u64 = 1_000_000;
const MAX_STALLED_INDICES_IN_QUEUE: u64 = 10;
const ORDER_QUEUE_INIT_CAPACITY: usize = 500;


pub type OrderProcessingResult = Vec<Result<Success, Failed>>;


#[derive(Debug)]
pub enum Success {
    Accepted {
        order_id: u64,
        order_type: OrderType,
        ts: SystemTime,
    },

    Filled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: f64,
        qty: f64,
        ts: SystemTime,
    },

    PartiallyFilled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: f64,
        qty: f64,
        ts: SystemTime,
    },

    Amended {
        order_id: u64,
        price: f64,
        qty: f64,
        ts: SystemTime,
    },

    Cancelled { order_id: u64, ts: SystemTime },
}


#[derive(Debug)]
pub enum Failed {
    ValidationFailed(String),
    DuplicateOrderID(u64),
    NoMatch(u64),
    OrderNotFound(u64),
}


pub struct Orderbook<Asset>
    where
        Asset: Debug + Clone + Copy + Eq,
{
    base_asset: Asset,
    quote_asset: Asset,
    bid_queue: OrderQueue<Order<Asset>>,
    ask_queue: OrderQueue<Order<Asset>>,
    seq: sequence::TradeSequence,
    order_validator: OrderRequestValidator<Asset>,
    db: DBWithThreadMode<SingleThreaded>
}


impl<Asset> Orderbook<Asset>
    where
        Asset: Debug + Clone + Copy + Eq,
{
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
    pub fn new(base_asset: Asset, quote_asset: Asset) -> Self {
        let path = "./db.rocks";
        let mut cf_opts = Options::default();
        cf_opts.set_max_write_buffer_number(16);
        let cf = ColumnFamilyDescriptor::new("cf1", cf_opts);
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        Orderbook {
            base_asset,
            quote_asset,
            bid_queue: OrderQueue::new(
                OrderSide::Bid,
                MAX_STALLED_INDICES_IN_QUEUE,
                ORDER_QUEUE_INIT_CAPACITY,
            ),
            ask_queue: OrderQueue::new(
                OrderSide::Ask,
                MAX_STALLED_INDICES_IN_QUEUE,
                ORDER_QUEUE_INIT_CAPACITY,
            ),
            seq: sequence::new_sequence_gen(MIN_SEQUENCE_ID, MAX_SEQUENCE_ID),
            order_validator: OrderRequestValidator::new(
                base_asset,
                quote_asset,
                MIN_SEQUENCE_ID,
                MAX_SEQUENCE_ID,
            ),
            db: DB::open_cf_descriptors(&db_opts, path, vec![cf]).unwrap()
        }
    }


    pub fn persist_result(&mut self, proc_result: &OrderProcessingResult) -> () {
        for result in proc_result {
            let id = match result {
                Ok(Success::Accepted{order_id, ..}) => { self.db.put(order_id.to_string(), "a"); order_id},
                Ok(Success::PartiallyFilled{order_id, qty, ..}) => { self.db.put(order_id.to_string(),"pf"); order_id},
                Ok(Success::Filled{order_id, qty, ..}) =>  { self.db.put(order_id.to_string(),"f"); order_id},
                _ => &0
            };
            // if self.db.key_may_exist(id.to_string()) {
            //     match self.db.get(id.to_string()) {
            //         Ok(Some(value)) => println!("retrieved value {}", String::from_utf8(value).unwrap()),
            //         Ok(None) => println!("value not found"),
            //         Err(e) => println!("operational problem encountered: {}", e),
            //     }
            // }
        }
    }



    pub fn process_order(&mut self, order: OrderRequest<Asset>) -> OrderProcessingResult {
        // processing result accumulator
        let mut proc_result: OrderProcessingResult = vec![];

        // validate requestç
        if let Err(reason) = self.order_validator.validate(&order) {
            proc_result.push(Err(Failed::ValidationFailed(String::from(reason))));
            return proc_result;
        }

        match order {
            OrderRequest::NewMarketOrder {
                base_asset,
                quote_asset,
                side,
                qty,
                ts: _ts,
            } => {
                // generate new ID for order
                let order_id = self.seq.next_id();
                proc_result.push(Ok(Success::Accepted {
                    order_id,
                    order_type: OrderType::Market,
                    ts: SystemTime::now(),
                }));

                self.process_market_order(
                    &mut proc_result,
                    order_id,
                    base_asset,
                    quote_asset,
                    side,
                    qty,
                );
            }

            OrderRequest::NewLimitOrder {
                base_asset,
                quote_asset: quote_asset,
                side,
                price,
                qty,
                ts,
            } => {
                let order_id = self.seq.next_id();
                proc_result.push(Ok(Success::Accepted {
                    order_id,
                    order_type: OrderType::Limit,
                    ts: SystemTime::now(),
                }));

                self.process_limit_order(
                    &mut proc_result,
                    order_id,
                    base_asset,
                    quote_asset,
                    side,
                    price,
                    qty,
                    ts,
                );
            }

            OrderRequest::AmendOrder {
                id,
                side,
                price,
                qty,
                ts,
            } => {
                self.process_order_amend(&mut proc_result, id, side, price, qty, ts);
            }

            OrderRequest::CancelOrder { id, side } => {
                self.process_order_cancel(&mut proc_result, id, side);
            }
        }

        // return collected processing results

        proc_result
    }


    /// Get current spread as a tuple: (bid, ask)
    pub fn current_spread(&mut self) -> Option<(f64, f64)> {
        let bid = self.bid_queue.peek()?.price;
        let ask = self.ask_queue.peek()?.price;
        Some((bid, ask))
    }


    /* Processing logic */

    fn process_market_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        base_asset: Asset,
        quote_asset: Asset,
        side: OrderSide,
        qty: f64,
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
                qty,
            );

            if !matching_complete {
                // match the rest
                self.process_market_order(
                    results,
                    order_id,
                    base_asset,
                    quote_asset,
                    side,
                    qty - opposite_order.qty,
                );
            }

        } else {
            // no limit orders found
            results.push(Err(Failed::NoMatch(order_id)));
        }
    }


    fn process_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        base_asset: Asset,
        quote_asset: Asset,
        side: OrderSide,
        price: f64,
        qty: f64,
        ts: SystemTime,
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
                    qty,
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
                        qty - opposite_order.qty,
                        ts,
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
                    qty,
                    ts,
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
                qty,
                ts,
            );
        }
    }


    fn process_order_amend(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        side: OrderSide,
        price: f64,
        qty: f64,
        ts: SystemTime,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        if order_queue.amend(
            order_id,
            price,
            ts,
            Order {
                order_id,
                base_asset: self.base_asset,
                quote_asset: self.quote_asset,
                side,
                price,
                qty,
            },
        )
        {
            results.push(Ok(Success::Amended {
                order_id: order_id,
                price,
                qty,
                ts: SystemTime::now(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }


    fn process_order_cancel(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        side: OrderSide,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        if order_queue.cancel(order_id) {
            results.push(Ok(Success::Cancelled {
                order_id: order_id,
                ts: SystemTime::now(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }


    /* Helpers */


    fn store_new_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: u64,
        base_asset: Asset,
        quote_asset: Asset,
        side: OrderSide,
        price: f64,
        qty: f64,
        ts: SystemTime,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };
        if !order_queue.insert(
            order_id,
            price,
            ts,
            Order {
                order_id,
                base_asset,
                quote_asset,
                side,
                price,
                qty,
            },
        )
        {
            results.push(Err(Failed::DuplicateOrderID(order_id)))
        };
    }


    fn order_matching(
        &mut self,
        results: &mut OrderProcessingResult,
        opposite_order: &Order<Asset>,
        order_id: u64,
        base_asset: Asset,
        quote_asset: Asset,
        order_type: OrderType,
        side: OrderSide,
        qty: f64,
    ) -> bool {

        // real processing time
        let deal_time = SystemTime::now();

        // match immediately
        if qty < opposite_order.qty {
            // fill new limit and modify opposite limit

            // report filled new order
            results.push(Ok(Success::Filled {
                order_id,
                side,
                order_type,
                price: opposite_order.price,
                qty,
                ts: deal_time,
            }));

            // report partially filled opposite limit order
            results.push(Ok(Success::PartiallyFilled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                qty,
                ts: deal_time,
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
                    qty: opposite_order.qty - qty,
                });
            }

        } else if qty > opposite_order.qty {
            // partially fill new limit order, fill opposite limit and notify to process the rest

            // report new order partially filled
            results.push(Ok(Success::PartiallyFilled {
                order_id,
                side,
                order_type,
                price: opposite_order.price,
                qty: opposite_order.qty,
                ts: deal_time,
            }));

            // report filled opposite limit order
            results.push(Ok(Success::Filled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                qty: opposite_order.qty,
                ts: deal_time,
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

        } else {
            // orders exactly match -> fill both and remove old limit

            // report filled new order
            results.push(Ok(Success::Filled {
                order_id,
                side,
                order_type,
                price: opposite_order.price,
                qty,
                ts: deal_time,
            }));
            // report filled opposite limit order
            results.push(Ok(Success::Filled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                qty,
                ts: deal_time,
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

        // complete matching
        true
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use super::super::orders;

    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    pub enum Asset {
        USD,
        BTC,
    }

    #[test]
    fn cancel_nonexisting() {
        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let request = orders::limit_order_cancel_request(1, OrderSide::Bid);
        let mut result = orderbook.process_order(request);

        assert_eq!(result.len(), 1);
        match result.pop().unwrap() {
            Err(_) => (),
            _ => panic!("unexpected events"),
        }
    }
}
