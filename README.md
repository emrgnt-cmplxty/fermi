# Introducing GDEX-CORE

GDEX-CORE hosts ongoing work on the GDEX order book blockchain.  The goal of this initial work is to successively approximate the functionality of a blockchain via a ground-up approach. The project is currently in an infant stage.  

Currently implemented are a basic order-matching engine, cryptographic primitives, and basic blockchain structures. 

Tests reside in suite/test, benchmarks reside in suite/benches

### Use


    # build
    cargo build --release

    # test
    cargo test

    # or, if you want to include batch features
    cargo bench

    # lastly, to bench just order book placement
    # delete the suite/db.rocks before running to make a clean db
    rm -rf suite/db.rocks && cargo bench place_orders


# Overview 


### How is the repo organized?

    gdex-core 
    ├── app                        # Toy consensus implementation
    ├── core                       # Blockchain primitives (i.e. block, transaction, vote_cert)
    ├── crypto                     # Cryptographic primitives (i.e. Ed25519 signature schema)
    ├── crypto-derive              # Cryptographic support functions
    ├── engine                     # Orderbook processing logic 
    ├── proc                       # Blockchain modules and support (i.e. bank, stake, ..) 
    ├── type                       # Internal type definitions
    └── suite                      # Bulk testing and benching

## Walkthrough 

### Order book Engine

The order book is implemented in /engine.  

Each instance of orderbook is a single-threaded reactive module for the certain currency pair. It consumes orders and return vector of events, generated during processing.

Supported features:

* market orders
* limit orders
* amending limit order price/quantity
* cancelling limit order
* partial filling

### Application Module(s)

Indiviual application modules are implemented in /proc.  

Parital support is currently implemented for asset balance, order book, and staking management. Controllers are implemented in dedicated classes and will be responsible for storing and updating internal state as transactions are submitted to the blockchain.

Supported features:

* asset creation
* asset transfer
* spot order book interaction
* asset staking

### Blockchain Primitives

Blockchain primitives are implemented in /core. 

Supported features:
* block
* transaction
* hash clock for eventual VDF implementation
* vote certificate for proving consensus

### Crypto & Crypto-Derive

Cryptographic primitives are implemented in /crypto.

Supported features:
* Ed25519 signature schema
* SHA3 hashing

### Other

Toy consensus in /app.
Testing and benching suite in /suite.
