extern crate engine;
extern crate rocksdb;
extern crate types;

use app::{
    router::{
        asset_creation_transaction, order_transaction, orderbook_creation_transaction, payment_transaction,
        route_transaction,
    },
    toy_consensus::ConsensusManager,
};
use core::{
    block::Block,
    hash_clock::HashClock,
    transaction::{TransactionRequest, TransactionVariant},
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use gdex_crypto::HashValue;
use proc::{account::generate_key_pair, bank::PRIMARY_ASSET_ID};
use rand::{rngs::ThreadRng, Rng};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread::{spawn, JoinHandle},
};
use types::{
    account::AccountPubKey, asset::AssetId, hash_clock::HashTime, orderbook::OrderSide,
};

const N_ORDERS_BENCH: u64 = 2_000;
const N_ACCOUNTS: u64 = 2_000;
// amount to transfer to accounts which participate in testing sequence
// transfer amount should be enough to cover 100 orders at the higher end
// potential order price
const TRANSFER_AMOUNT: u64 = 100_000_000;
const BASE_ASSET_ID: AssetId = PRIMARY_ASSET_ID;
const QUOTE_ASSET_ID: AssetId = 1;

fn get_last_block(consensus_manager: &ConsensusManager) -> Block<TransactionVariant> {
    let blocks: &Vec<Block<TransactionVariant>> = consensus_manager.get_block_container().get_blocks();
    assert!(blocks.len() > 0);
    blocks[blocks.len() - 1].clone()
}

fn get_clock_handler(consensus_manager: &ConsensusManager, block: &Block<TransactionVariant>) -> JoinHandle<HashValue> {
    let hash_clock: Arc<Mutex<HashClock>> = Arc::new(Mutex::new(HashClock::new(
        block.get_hash_time(),
        consensus_manager.get_ticks_per_cycle(),
    )));
    spawn(move || {
        let mut hash_clock: MutexGuard<HashClock> = hash_clock.lock().unwrap();
        hash_clock.cycle();
        hash_clock.get_hash_time()
    })
}

fn place_orders_consensus(
    consensus_manager: &mut ConsensusManager,
    transactions: Vec<TransactionRequest<TransactionVariant>>,
) {
    // require and load the previous block
    let last_block: Block<TransactionVariant> = get_last_block(&consensus_manager);

    // begin ticking hash clock in async thread
    let hash_time_handler: JoinHandle<HashValue> = get_clock_handler(&consensus_manager, &last_block);

    // verify transactions and update state
    for order_transaction in transactions.iter() {
        route_transaction(consensus_manager, order_transaction).unwrap();
    }

    // propose & validate new block
    let new_hash_time: HashTime = hash_time_handler.join().unwrap();
    let new_block: Block<TransactionVariant> = consensus_manager.propose_block(transactions, new_hash_time).unwrap();
    consensus_manager
        .validate_and_store_block(new_block, last_block)
        .unwrap();
}

#[cfg(feature = "batch")]
mod batch_benches {
    use super::*;
    use app::router::batch_functions::{route_transaction_batch, route_transaction_batch_multithreaded};

    pub fn place_orders_consensus_batch(
        consensus_manager: &mut ConsensusManager,
        transactions: Vec<TransactionRequest<TransactionVariant>>,
    ) {
        // require and load the previous block
        let last_block: Block<TransactionVariant> = get_last_block(&consensus_manager);

        // begin ticking hash clock in async thread
        let hash_time_handler: JoinHandle<HashTime> = get_clock_handler(&consensus_manager, &last_block);

        // verify transactions and update state
        route_transaction_batch(consensus_manager, &transactions).unwrap();

        // fetch hash time for inclusion with new block
        let new_hash_time: HashTime = hash_time_handler.join().unwrap();

        // propose and validate new block
        let new_block: Block<TransactionVariant> =
            consensus_manager.propose_block(transactions, new_hash_time).unwrap();
        consensus_manager
            .validate_and_store_block(new_block, last_block)
            .unwrap();
    }

    pub fn place_orders_consensus_batch_multithreaded(
        consensus_manager: &mut ConsensusManager,
        transactions: Vec<TransactionRequest<TransactionVariant>>,
        n_threads: u64,
    ) {
        // require and load the previous block
        let last_block: Block<TransactionVariant> = get_last_block(&consensus_manager);

        // begin ticking hash clock in async thread
        let hash_time_handler: JoinHandle<HashTime> = get_clock_handler(&consensus_manager, &last_block);

        // verify transactions and update state
        route_transaction_batch_multithreaded(consensus_manager, &transactions, n_threads).unwrap();

        // fetch hash time for inclusion with new block
        let new_hash_time: HashTime = hash_time_handler.join().unwrap();

        // propose and validate new block
        let new_block: Block<TransactionVariant> =
            consensus_manager.propose_block(transactions, new_hash_time).unwrap();
        consensus_manager
            .validate_and_store_block(new_block, last_block)
            .unwrap();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng: ThreadRng = rand::thread_rng();

    // initialize consensus manager
    let mut consensus_manager: ConsensusManager = ConsensusManager::new();
    let validator_pub_key: AccountPubKey = consensus_manager.get_validator_pub_key();

    // initiate new consensus instances by creating the genesis block from perspective of primary validator
    let genesis_block: Block<TransactionVariant> = consensus_manager.build_genesis_block().unwrap();

    // validate block immediately as genesis proposer is only staked validator
    consensus_manager
        .store_genesis_block(
            genesis_block,
        );

    let signed_transaction: TransactionRequest<TransactionVariant> =
        asset_creation_transaction(validator_pub_key, consensus_manager.get_validator_private_key()).unwrap();
    route_transaction(&mut consensus_manager, &signed_transaction).unwrap();

    // create quote asset, this is asset #1 of the blockchain
    let signed_transaction: TransactionRequest<TransactionVariant> =
        asset_creation_transaction(validator_pub_key, consensus_manager.get_validator_private_key()).unwrap();
    route_transaction(&mut consensus_manager, &signed_transaction).unwrap();

    // create orderbook, the base asset for this orderbook will be the primary asset (# 0) created at genesis
    let signed_transaction: TransactionRequest<TransactionVariant> = orderbook_creation_transaction(
        validator_pub_key,
        consensus_manager.get_validator_private_key(),
        BASE_ASSET_ID,
        QUOTE_ASSET_ID,
    )
    .unwrap();
    route_transaction(&mut consensus_manager, &signed_transaction).unwrap();

    // generate and fund accounts and create orders for bench
    let mut i_account: u64 = 0;
    let mut bench_transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();

    while i_account < N_ACCOUNTS {
        // generate account
        let (account_pub_key, account_priv_key) = generate_key_pair();

        // fund new account with base asset
        let signed_transaction: TransactionRequest<TransactionVariant> = payment_transaction(
            validator_pub_key,
            consensus_manager.get_validator_private_key(),
            account_pub_key,
            BASE_ASSET_ID,
            TRANSFER_AMOUNT,
        )
        .unwrap();
        route_transaction(&mut consensus_manager, &signed_transaction).unwrap();

        // fund new account with quote asset
        let signed_transaction: TransactionRequest<TransactionVariant> = payment_transaction(
            validator_pub_key,
            consensus_manager.get_validator_private_key(),
            account_pub_key,
            QUOTE_ASSET_ID,
            TRANSFER_AMOUNT,
        )
        .unwrap();
        route_transaction(&mut consensus_manager, &signed_transaction).unwrap();

        // create and store an orderbook transaction for future execution
        let order_side: OrderSide = if i_account % 2 == 0 {
            OrderSide::Bid
        } else {
            OrderSide::Ask
        };
        let quantity: u64 = rng.gen_range(1, 100);
        let price: u64 = rng.gen_range(1, 100);
        let signed_transaction: TransactionRequest<TransactionVariant> = order_transaction(
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

    let mut group = c.benchmark_group("toy_consensus");
    group.throughput(Throughput::Elements((N_ORDERS_BENCH) as u64));

    group.bench_function("place_orders_toy_consensus", |b| {
        b.iter(|| place_orders_consensus(&mut consensus_manager, bench_transactions.clone()))
    });

    #[cfg(feature = "batch")]
    {
        group.bench_function("place_orders_toy_consensus_batch_vanilla", |b| {
            b.iter(|| batch_benches::place_orders_consensus_batch(&mut consensus_manager, bench_transactions.clone()))
        });

        group.bench_function("place_orders_toy_consensus_batch_1_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_consensus_batch_multithreaded(
                    &mut consensus_manager,
                    bench_transactions.clone(),
                    1,
                )
            })
        });

        group.bench_function("place_orders_toy_consensus_batch_2_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_consensus_batch_multithreaded(
                    &mut consensus_manager,
                    bench_transactions.clone(),
                    2,
                )
            })
        });

        group.bench_function("place_orders_toy_consensus_batch_4_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_consensus_batch_multithreaded(
                    &mut consensus_manager,
                    bench_transactions.clone(),
                    4,
                )
            })
        });

        group.bench_function("place_orders_toy_consensus_batch_8_threads", |b| {
            b.iter(|| {
                batch_benches::place_orders_consensus_batch_multithreaded(
                    &mut consensus_manager,
                    bench_transactions.clone(),
                    8,
                )
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
