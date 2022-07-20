extern crate rocksdb;
extern crate engine;
extern crate types;

use app::{
    router::{asset_creation_txn, orderbook_creation_txn, order_transaction, payment_txn, route_transaction},
    toy_consensus::ConsensusManager,
};
use core::{
    transaction::{
        TxnRequest, 
        TxnVariant, 
    },
};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use gdex_crypto::traits::{Uniform};
use types::{
    asset::AssetId,
    account::{AccountPubKey, AccountPrivKey},
    orderbook::OrderSide,
};
use rand::{Rng, rngs::ThreadRng};

const N_ORDERS_BENCH: u64 = 1_024;
const N_ACCOUNTS: u64 = 1_024;
const TRANSFER_AMOUNT: u64 = 500_000_000;
const BASE_ASSET_ID: AssetId = 0;
const QUOTE_ASSET_ID: AssetId = 1;

fn place_orders_consensus(
    consensus_manager: &mut ConsensusManager,
    transactions: Vec<TxnRequest<TxnVariant>>,
) 
{
    for order_transaction in transactions.iter() {
        route_transaction(consensus_manager, &order_transaction).unwrap();
    }
    consensus_manager.propose_block(transactions).unwrap();
}


#[cfg(feature = "batch")]
use app::router::route_transaction_batch;

#[cfg(feature = "batch")]
fn place_orders_consensus_batch(
    consensus_manager: &mut ConsensusManager,
    transactions: Vec<TxnRequest<TxnVariant>>,
) 
{
    route_transaction_batch(consensus_manager, &transactions).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng: ThreadRng = rand::thread_rng();

    // initialize consensus manager
    let mut consensus_manager: ConsensusManager = ConsensusManager::new();
    let primary_pub_key: AccountPubKey = consensus_manager.get_validator_pub_key();

    // initiate new consensus by creating the genesis block from perspective of primary validator
    consensus_manager.build_genesis_block().unwrap();
    let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(primary_pub_key, &consensus_manager.get_validator_private_key()).unwrap();
    route_transaction(&mut consensus_manager, &signed_txn).unwrap();

    // create base asset & orderbook
    let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(primary_pub_key, &consensus_manager.get_validator_private_key()).unwrap();
    route_transaction(&mut consensus_manager, &signed_txn).unwrap();

    let signed_txn: TxnRequest<TxnVariant> = orderbook_creation_txn(primary_pub_key, &consensus_manager.get_validator_private_key(), BASE_ASSET_ID, QUOTE_ASSET_ID).unwrap();
    route_transaction(&mut consensus_manager, &signed_txn).unwrap();

    // generate and fund accounts for benchmarking
    let mut i_account: u64 = 0;
    let mut bench_transactions: Vec<TxnRequest<TxnVariant>> = Vec::new();

    while i_account < N_ACCOUNTS - 1 {
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();
        
        // fund new account with base asset
        let signed_txn: TxnRequest<TxnVariant> = payment_txn(
            primary_pub_key,
            consensus_manager.get_validator_private_key(),
            account_pub_key,
            BASE_ASSET_ID,
            TRANSFER_AMOUNT
        ).unwrap();
        route_transaction(&mut consensus_manager, &signed_txn).unwrap();
        
        // fund new account with quote asset
        let signed_txn: TxnRequest<TxnVariant> = payment_txn(
            primary_pub_key,
            consensus_manager.get_validator_private_key(),
            account_pub_key,
            QUOTE_ASSET_ID,
            TRANSFER_AMOUNT
        ).unwrap();
        route_transaction(&mut consensus_manager, &signed_txn).unwrap();

        // create and store an orderbook transaction for future execution
        let order_side: OrderSide = if i_account % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        let qty: u64 = rng.gen_range(1, 100);
        let price: u64 = rng.gen_range(1, 100);
        let signed_txn: TxnRequest<TxnVariant>  = order_transaction(
            primary_pub_key, 
            &consensus_manager.get_validator_private_key(), 
            BASE_ASSET_ID, 
            QUOTE_ASSET_ID,
            order_side,
            price,
            qty
        ).unwrap();
        // create initial asset
        bench_transactions.push(signed_txn);

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
