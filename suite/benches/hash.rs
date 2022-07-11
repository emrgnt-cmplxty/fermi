// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate criterion;

use criterion::Criterion;

use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use rand::{prelude::ThreadRng, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, CryptoHasher, BCSCryptoHash, Serialize, Deserialize)]
pub struct TestDiemCrypto(pub String);

use diem_crypto::{
    hash::CryptoHash,
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    traits::{Signature, SigningKey, Uniform},
};


// make a new struct for an order that we have to hash
#[derive(Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
struct Order {
    quantity: i32,
    side: String,
}

// we pass in the number of messages we want to verify, a list of signatures, a list of private keys, and a list of public keys
#[inline]
fn basic_hash(
    order: &Order,
) {
    // getting the hash the first time
    let mut i = 0;
    while i < 10_000{
        let hash1 = order.hash();
        i+=1
    }
}


fn criterion_benchmark(c: &mut Criterion) {
    
    let order = Order {
        quantity: 10,
        side: String::from("Buy"),
    };

    c.bench_function("basic_hash", |b| {
        b.iter(|| basic_hash(&order))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);