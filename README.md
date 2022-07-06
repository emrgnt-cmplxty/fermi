# GDEX Rust Orderbook Research

The project currently consists of a basic order-matching engine (engine), account management process (proc), RocksDB database implementation, and cryptographic signature library (crypto).

Tests reside in suite/test, benchmarks reside in suite/benches

## Use


    # build
    cargo build --release

    # test
    cargo test

    # bench
    cargo bench


## Orderbook Details

Each instance of orderbook is a single-threaded reactive module for the certain currency pair. It consumes orders and return vector of events, generated during processing.

Supported features:

* market orders
* limit orders
* amending limit order price/quantity
* cancelling limit order
* partial filling


## Account Management Details

The AccountController creates a balance gated implementation of the orderbook.  The controller posts orders and modifies relevant account states after each order.


Supported features:

* basic balance gating
* balance updating
* TODO - cryptopgrahic verification


## Example output
Full example code could be found in `bin/example.rs` of initial commit. Here is an event log created from processing test orders:

```
Order => NewLimitOrder { base_asset: BTC, quote_asset: USD, side: Bid, price: 0.98, qty: 5.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859954000 } }
Processing => [Ok(Accepted { order_id 1, order_type: Limit, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860016000 } })]
Spread => not available

Order => NewLimitOrder { base_asset: BTC, quote_asset: USD, side: Ask, price: 1.02, qty: 1.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859954000 } }
Processing => [Ok(Accepted { order_id 2, order_type: Limit, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860064000 } })]
Spread => border_id 0.98, ask: 1.02

Order => AmendOrder { order_id 1, side: Bid, price: 0.99, qty: 4.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859954000 } }
Processing => [Ok(Amended { order_id 1, price: 0.99, qty: 4.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860094000 } })]
Spread => border_id 0.99, ask: 1.02

Order => NewLimitOrder { base_asset: BTC, quote_asset: USD, side: Bid, price: 1.01, qty: 0.4, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859955000 } }
Processing => [Ok(Accepted { order_id 3, order_type: Limit, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860119000 } })]
Spread => border_id 1.01, ask: 1.02

Order => NewLimitOrder { base_asset: BTC, quote_asset: USD, side: Ask, price: 1.03, qty: 0.5, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859955000 } }
Processing => [Ok(Accepted { order_id 4, order_type: Limit, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860155000 } })]
Spread => border_id 1.01, ask: 1.02

Order => NewMarketOrder { base_asset: BTC, quote_asset: USD, side: Bid, qty: 1.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859955000 } }
Processing => [Ok(Accepted { order_id 5, order_type: Market, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860180000 } }), Ok(Filled { order_order_id 5, side: Bid, order_type: Market, price: 1.02, qty: 1.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860183000 } }), Ok(Filled { order_order_id 2, side: Ask, order_type: Limit, price: 1.02, qty: 1.0, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860183000 } })]
Spread => border_id 1.01, ask: 1.03

Order => NewLimitOrder { base_asset: BTC, quote_asset: USD, side: Ask, price: 1.05, qty: 0.5, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859955000 } }
Processing => [Ok(Accepted { order_id 6, order_type: Limit, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860248000 } })]
Spread => border_id 1.01, ask: 1.03

Order => CancelOrder { order_id 4, side: Ask }
Processing => [Ok(Cancelled { order_id 4, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860291000 } })]
Spread => border_id 1.01, ask: 1.05

Order => NewLimitOrder { base_asset: BTC, quote_asset: USD, side: Bid, price: 1.06, qty: 0.6, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 859955000 } }
Processing => [Ok(Accepted { order_id 7, order_type: Limit, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860320000 } }), Ok(PartiallyFilled { order_order_id 7, side: Bid, order_type: Limit, price: 1.05, qty: 0.5, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860325000 } }), Ok(Filled { order_order_id 6, side: Ask, order_type: Limit, price: 1.05, qty: 0.5, ts: SystemTime { tv_sec: 1516040690, tv_nsec: 860325000 } })]
Spread => not available
```
# rust-orderbook
