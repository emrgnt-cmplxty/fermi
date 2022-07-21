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
    hash_clock::{HashClock, HASH_TIME_INIT_MSG},
    transaction::{TransactionRequest, TransactionVariant},
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use gdex_crypto::{hash::CryptoHash, traits::Uniform, HashValue};
use proc::bank::PRIMARY_ASSET_ID;
use rand::{rngs::ThreadRng, Rng};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread::{spawn, JoinHandle},
};
use types::{
    account::{AccountPrivKey, AccountPubKey},
    asset::AssetId,
    hash_clock::HashTime,
    orderbook::OrderSide,
    spot::DiemCryptoMessage,
};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;
// amount to transfer to accounts which participate in testing sequence
// transfer amount should be enough to cover 100 orders at the higher end
// potential order price
const TRANSFER_AMOUNT: u64 = 500_000_000;
const BASE_ASSET_ID: AssetId = PRIMARY_ASSET_ID;
const QUOTE_ASSET_ID: AssetId = 1;

fn place_orders_consensus(
    consensus_manager: &mut ConsensusManager,
    transactions: Vec<TransactionRequest<TransactionVariant>>,
) {
    // require and load the previous block
    let blocks: &Vec<Block<TransactionVariant>> = consensus_manager.get_block_container().get_blocks();
    assert!(blocks.len() > 0);
    let block: Block<TransactionVariant> = blocks[blocks.len() - 1].clone();

    // begin ticking hash clock in async thread
    let hash_time_handler: JoinHandle<HashValue> = get_clock_handler(&consensus_manager, &block);

    // verify transactions and update state
    for order_transaction in transactions.iter() {
        route_transaction(consensus_manager, order_transaction).unwrap();
    }

    // propose & validate new block
    let new_hash_time: HashTime = hash_time_handler.join().unwrap();
    let new_block: Block<TransactionVariant> = consensus_manager.propose_block(transactions, new_hash_time).unwrap();
    consensus_manager
        .validate_and_store_block(new_block, block.get_hash_time())
        .unwrap();
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

#[cfg(feature = "batch")]
use app::router::route_transaction_batch;

#[cfg(feature = "batch")]
fn place_orders_consensus_batch(
    consensus_manager: &mut ConsensusManager,
    transactions: Vec<TransactionRequest<TransactionVariant>>,
) {
    // require and load the previous block
    let blocks: &Vec<Block<TransactionVariant>> = consensus_manager.get_block_container().get_blocks();
    assert!(blocks.len() > 0);
    let block: Block<TransactionVariant> = blocks[blocks.len() - 1].clone();

    // begin ticking hash clock in async thread
    let hash_time_handler: JoinHandle<HashTime> = get_clock_handler(&consensus_manager, &block);

    // verify transactions and update state
    route_transaction_batch(consensus_manager, &transactions).unwrap();

    // fetch hash time for inclusion with new block
    let new_hash_time: HashTime = hash_time_handler.join().unwrap();
    // propose and validate new block
    let new_block: Block<TransactionVariant> = consensus_manager.propose_block(transactions, new_hash_time).unwrap();
    consensus_manager
        .validate_and_store_block(new_block, block.get_hash_time())
        .unwrap();
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
        .validate_and_store_block(genesis_block, DiemCryptoMessage(HASH_TIME_INIT_MSG.to_string()).hash())
        .unwrap();

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

    while i_account < N_ACCOUNTS - 1 {
        let account_priv_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&account_priv_key).into();

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
        // create initial asset
        bench_transactions.push(signed_transaction);

        i_account += 1;
    }

    let mut group = c.benchmark_group("toy_consensus");
    group.throughput(Throughput::Elements((N_ORDERS_BENCH) as u64));

    group.bench_function("place_orders_toy_consensus", |b| {
        b.iter(|| place_orders_consensus(&mut consensus_manager, bench_transactions.clone()))
    });

    #[cfg(feature = "batch")]
    group.bench_function("place_orders_toy_consensus_batch", |b| {
        b.iter(|| place_orders_consensus_batch(&mut consensus_manager, bench_transactions.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
