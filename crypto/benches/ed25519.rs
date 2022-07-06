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
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    traits::{Signature, SigningKey, Uniform},
};

const NUMBER_OF_MESSAGES: i32 = 100000;
const NUMBER_OF_ACCOUNTS: i32 = 100;

// we pass in the number of messages we want to verify, a list of signatures, a list of private keys, and a list of public keys
#[inline]
fn only_verify_signatures(
    sigs: &mut Vec<Ed25519Signature>,
    public_keys: &mut Vec<Ed25519PublicKey>,
    possible_messages: &mut Vec<TestDiemCrypto>,
) {
    let messages_per_account = NUMBER_OF_MESSAGES / NUMBER_OF_ACCOUNTS;
    for i in 0..NUMBER_OF_ACCOUNTS {
        let public_key = &public_keys[i as usize];
        for x in 0..messages_per_account {
            let sig = &sigs[((i * messages_per_account) + x) as usize];
            let msg = &possible_messages[x as usize];
            sig.verify(msg, &public_key);
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let messages_per_account = NUMBER_OF_MESSAGES / NUMBER_OF_ACCOUNTS;
    // getting list of possible raw messages
    let messages_pre_format: Vec<i32> = (0..messages_per_account).collect();
    // getting formatted list of possible messages
    let mut messages: Vec<TestDiemCrypto> = messages_pre_format
        .iter()
        .map(|&x| TestDiemCrypto(format!("{}", x).to_string()))
        .collect();
    // instantiating private keys
    let mut private_keys: Vec<Ed25519PrivateKey> = Vec::new();
    // instantiatng public keys
    let mut public_keys: Vec<Ed25519PublicKey> = Vec::new();
    // instantiating signatures
    let mut sigs: Vec<Ed25519Signature> = Vec::new();
    // looping through accounts
    for _i in 0..NUMBER_OF_ACCOUNTS {
        let mut csprng: ThreadRng = thread_rng();
        let private_key = Ed25519PrivateKey::generate(&mut csprng);
        let public_key: Ed25519PublicKey = (&private_key).into();
        // pushing transactions
        for x in 0..messages_per_account {
            let msg = TestDiemCrypto(format!("{}", x).to_string());
            let sig: Ed25519Signature = private_key.sign(&msg);
            sigs.push(sig);
        }
        // adding keys to vector
        private_keys.push(private_key);
        public_keys.push(public_key);
    }
    c.bench_function("only_verify_signatures", |b| {
        b.iter(|| only_verify_signatures(&mut sigs, &mut public_keys, &mut messages))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
