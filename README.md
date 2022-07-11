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
    # or, if you want to include batch features
    cargo bench --features="batch"
    # lastly, to bench just order book placement
    cargo bench --features="batch" place_orders

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

## How is this module organized?

    crypto # In-memory storage of blocks and related data structures
    ├── src
    │   ├── block_storage          # In-memory storage of blocks and related data structures
    │   ├── consensusdb            # Database interaction to persist consensus data for safety and liveness
    │   ├── liveness               # RoundState, proposer, and other liveness related code
    │   └── test_utils             # Mock implementations that are used for testing only
    ├── consensus-types            # Consensus data types (i.e. quorum certificates)
    └── safety-rules               # Safety (voting) rules
