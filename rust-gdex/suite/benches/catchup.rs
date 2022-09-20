// Copyright (c) 2022, BTI
// SPDX-License-Identifier: Apache-2.0
// to run this code, run cargo bench mutex_lock, for ex.
// TODO - cleanup this benchmark file

extern crate criterion;

use criterion::*;

use gdex_controller::{
    bank::controller::BankController,
    controller::Controller
};
use fastcrypto::{generate_production_keypair, traits::KeyPair as _};
use narwhal_crypto::KeyPair;

use std::sync::Arc;
use std::sync::Mutex;

fn criterion_benchmark(c: &mut Criterion) {

    // setup
    let n_users: usize = 1_000_000;
    let n_assets: usize = 10;
    let mut keypairs: Vec<KeyPair> = Vec::new();
    let mut bank_controller = BankController::default();
    for i in 0..n_users {
        let keypair = generate_production_keypair::<KeyPair>();
        if i % 1000 == 0 {
            println!("{}", i);
        }
        bank_controller.create_account(keypair.public()).unwrap();
        if i < n_assets {
            bank_controller.create_asset(keypair.public()).unwrap();
        };
        keypairs.push(keypair);
    }

    for i in 0..n_assets {
        let sender_kp = &keypairs[i];
        for j in 0..n_users {
            let receiver_kp = &keypairs[j];
            bank_controller
                .transfer(sender_kp.public(), receiver_kp.public(), i as u64, 1)
                .unwrap();
        }
    }

    let bank_controller = Arc::new(Mutex::new(bank_controller));

    c.bench_function("create_catchup_state", |b| {
        b.iter(|| {
            BankController::create_catchup_state(bank_controller.clone(), 0)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
