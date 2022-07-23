extern crate engine;
extern crate rocksdb;
extern crate types;

use app::{
    router::{
        asset_creation_transaction, order_transaction, orderbook_creation_transaction, payment_transaction,
        route_transaction,
    },
    validator::ValidatorController,
};
use core::{
    block::Block,
    hash_clock::HashClock,
    transaction::{TransactionRequest, TransactionVariant},
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use gdex_crypto::HashValue;
use proc::{account::generate_key_pair, bank::PRIMARY_ASSET_ID};
use rand::{Rng};
use std::{
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
};
use types::{asset::AssetId, orderbook::OrderSide};

const N_ORDERS_BENCH: u64 = 2_000;
const N_ACCOUNTS: u64 = 2_000;
// amount to transfer to accounts which participate in testing sequence
// transfer amount should be enough to cover 100 orders at the higher end
// potential order price
const TRANSFER_AMOUNT: u64 = 100_000_000;
const BASE_ASSET_ID: AssetId = PRIMARY_ASSET_ID;
const QUOTE_ASSET_ID: AssetId = 1;

fn get_last_block(validator_controller: &ValidatorController) -> Block<TransactionVariant> {
    let blocks = validator_controller.get_block_container().get_blocks();
    assert!(!blocks.is_empty());
    blocks[blocks.len() - 1].clone()
}

fn get_clock_handler(
    validator_controller: &ValidatorController,
    block: &Block<TransactionVariant>,
) -> JoinHandle<HashValue> {
    let hash_clock = Arc::new(Mutex::new(HashClock::new(
        block.get_hash_time(),
        validator_controller.get_ticks_per_cycle(),
    )));
    hash_clock
        .lock()
        .unwrap()
        .update_hash_time(block.get_hash_time(), block.get_block_hash());

    spawn(move || {
        let mut hash_clock = hash_clock.lock().unwrap();
        hash_clock.cycle();
        hash_clock.get_hash_time()
    })
}

fn place_orders_validator(
    validator_controller: &mut ValidatorController,
    transactions: Vec<TransactionRequest<TransactionVariant>>,
) {
    // require and load the previous block
    let last_block = get_last_block(validator_controller);

    // begin ticking hash clock in async thread
    let hash_time_handler = get_clock_handler(validator_controller, &last_block);

    // verify transactions and update state
    for order_transaction in transactions.iter() {
        route_transaction(validator_controller, order_transaction).unwrap();
    }

    // propose & validate new block
    let new_hash_time = hash_time_handler.join().unwrap();
    let new_block = validator_controller.propose_block(transactions, new_hash_time).unwrap();
    validator_controller
        .validate_and_store_block(new_block, last_block)
        .unwrap();
}

#[cfg(feature = "batch")]
mod batch_benches {
    use super::*;
    use app::router::batch_functions::{route_transaction_batch, route_transaction_batch_multithreaded};

    pub fn place_orders_validator_batch(
        validator_controller: &mut ValidatorController,
        transactions: Vec<TransactionRequest<TransactionVariant>>,
    ) {
        // require and load the previous block
        let last_block = get_last_block(validator_controller);

        // begin ticking hash clock in async thread
        let hash_time_handler = get_clock_handler(validator_controller, &last_block);

        // verify transactions and update state
        route_transaction_batch(validator_controller, &transactions).unwrap();

        // fetch hash time for inclusion with new block
        let new_hash_time = hash_time_handler.join().unwrap();

        // propose and validate new block
        let new_block = validator_controller.propose_block(transactions, new_hash_time).unwrap();
        validator_controller
            .validate_and_store_block(new_block, last_block)
            .unwrap();
    }

    pub fn place_orders_validator_batch_multithreaded(
        validator_controller: &mut ValidatorController,
        transactions: Vec<TransactionRequest<TransactionVariant>>,
        n_threads: u64,
    ) {
        // require and load the previous block
        let last_block = get_last_block(validator_controller);

        // begin ticking hash clock in async thread
        let hash_time_handler = get_clock_handler(validator_controller, &last_block);

        // verify transactions and update state
        route_transaction_batch_multithreaded(validator_controller, &transactions, n_threads).unwrap();

        // fetch hash time for inclusion with new block
        let new_hash_time = hash_time_handler.join().unwrap();

        // propose and validate new block
        let new_block = validator_controller.propose_block(transactions, new_hash_time).unwrap();
        validator_controller
            .validate_and_store_block(new_block, last_block)
            .unwrap();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    // initialize validator manager
    let mut validator_controller = ValidatorController::new();
    let pub_key = validator_controller.get_pub_key();

    // initiate new validator instances by creating the genesis block from perspective of primary validator
    let genesis_block = validator_controller.build_genesis_block().unwrap();

    // validate block immediately as genesis proposer is only staked validator
    validator_controller.store_genesis_block(genesis_block);

    let signed_transaction = asset_creation_transaction(pub_key, validator_controller.get_private_key()).unwrap();
    route_transaction(&mut validator_controller, &signed_transaction).unwrap();

    // create quote asset, this is asset #1 of the blockchain
    let signed_transaction = asset_creation_transaction(pub_key, validator_controller.get_private_key()).unwrap();
    route_transaction(&mut validator_controller, &signed_transaction).unwrap();

    // create orderbook, the base asset for this orderbook will be the primary asset (# 0) created at genesis
    let signed_transaction = orderbook_creation_transaction(
        pub_key,
        validator_controller.get_private_key(),
        BASE_ASSET_ID,
        QUOTE_ASSET_ID,
    )
    .unwrap();
    route_transaction(&mut validator_controller, &signed_transaction).unwrap();

    // generate and fund accounts and create orders for bench
    let mut i_account = 0;
    let mut bench_transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();

    while i_account < N_ACCOUNTS {
        // generate account
        let (account_pub_key, account_priv_key) = generate_key_pair();

        // fund new account with base asset
        let signed_transaction = payment_transaction(
            pub_key,
            validator_controller.get_private_key(),
            account_pub_key,
            BASE_ASSET_ID,
            TRANSFER_AMOUNT,
        )
        .unwrap();
        route_transaction(&mut validator_controller, &signed_transaction).unwrap();

        // fund new account with quote asset
        let signed_transaction = payment_transaction(
            pub_key,
            validator_controller.get_private_key(),
            account_pub_key,
            QUOTE_ASSET_ID,
            TRANSFER_AMOUNT,
        )
        .unwrap();
        route_transaction(&mut validator_controller, &signed_transaction).unwrap();

        // create and store an orderbook transaction for future execution
        let order_side = if i_account % 2 == 0 {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        let quantity = rng.gen_range(1, 100);
        let price = rng.gen_range(1, 100);
        let signed_transaction = order_transaction(
            account_pub_key,
            &account_priv_key,
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            order_side,
            price,
            quantity,
        )
        .unwrap();

        // save transaction and continue
        bench_transactions.push(signed_transaction);
        i_account += 1;
    }

    let mut group = c.benchmark_group("toy_validator");
    group.throughput(Throughput::Elements((N_ORDERS_BENCH) as u64));

    group.bench_function("place_orders_toy_validator", |b| {
        b.iter(|| place_orders_validator(&mut validator_controller, bench_transactions.clone()))
    });

    #[cfg(feature = "batch")]
    {
        group.bench_function("place_orders_toy_validator_batch_vanilla", |b| {
            b.iter(|| {
                batch_benches::place_orders_validator_batch(&mut validator_controller, bench_transactions.clone())
            })
        });

        group.bench_function("place_orders_toy_validator_batch_1_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_validator_batch_multithreaded(
                    &mut validator_controller,
                    bench_transactions.clone(),
                    1,
                )
            })
        });

        group.bench_function("place_orders_toy_validator_batch_2_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_validator_batch_multithreaded(
                    &mut validator_controller,
                    bench_transactions.clone(),
                    2,
                )
            })
        });

        group.bench_function("place_orders_toy_validator_batch_4_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_validator_batch_multithreaded(
                    &mut validator_controller,
                    bench_transactions.clone(),
                    4,
                )
            })
        });

        group.bench_function("place_orders_toy_validator_batch_8_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_validator_batch_multithreaded(
                    &mut validator_controller,
                    bench_transactions.clone(),
                    8,
                )
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
