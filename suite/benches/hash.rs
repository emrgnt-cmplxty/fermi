// Copyright (c) The GDEX Core Contributors
// SPDX-License-Identifier: Apache-2.0
#[macro_use]
extern crate criterion;

use criterion::{Criterion, Throughput};
use gdex_crypto::hash::CryptoHash;
use gdex_crypto_derive::{BCSCryptoHash, CryptoHasher};
use serde::{Deserialize, Serialize};

// make a new struct for an order that we have to hash
#[derive(Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
struct Order {
    quantity: i32,
    side: String,
}
const NUMBER_OF_MESSAGES: u64 = 10_000;

// we pass in the number of messages we want to verify, a list of signatures, a list of private keys, and a list of public keys
#[inline]
fn basic_hash(order: &Order) {
    // getting the hash the first time
    let mut i = 0;
    while i < NUMBER_OF_MESSAGES {
        order.hash();
        i += 1
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let order = Order {
        quantity: 10,
        side: String::from("Buy"),
    };
    let mut group = c.benchmark_group("hash_speed");
    group.throughput(Throughput::Elements((NUMBER_OF_MESSAGES) as u64));
    group.bench_function("basic_hash", |b| b.iter(|| basic_hash(&order)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
