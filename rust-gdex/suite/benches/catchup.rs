// Copyright (c) 2022, BTI
// SPDX-License-Identifier: Apache-2.0
// to run this code, run cargo bench mutex_lock, for ex.
// TODO - cleanup this benchmark file

extern crate criterion;

use criterion::*;

use gdex_controller::{
    bank::controller::BankController,
    controller::Controller,
    event_manager::EventManager,
    spot::{
        controller::{SpotController, SPOT_CONTROLLER_ACCOUNT_PUBKEY},
        proto::LimitOrderRequest,
    },
    utils::engine::order_book::OrderBookWrapper,
};
// gdex
use fastcrypto::{generate_production_keypair, traits::KeyPair as _};
use gdex_types::{account::AccountPubKey, crypto::ToFromBytes, order_book::OrderSide};
use narwhal_crypto::KeyPair;

use std::collections::HashMap;
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

    let bank_controller_clone = bank_controller.clone();

    c.bench_function("catchup_bank_get_catchup_state", |b| {
        b.iter(|| bank_controller_clone.get_catchup_state())
    });

    c.bench_function("catchup_bank_controller_clone", |b| {
        b.iter(|| bank_controller_clone.clone())
    });

    // setup orderbooks
    let n_users = 100_000;
    let n_orderbooks = 10;

    // initialize the bank controller + spot controller with assets
    let bank_controller = Arc::new(Mutex::new(BankController::default()));
    let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
    // create bank account for spot controller account
    bank_controller
        .lock()
        .unwrap()
        .create_account(&controller_account)
        .unwrap();
    let orderbooks = HashMap::new();
    let spot_controller = Arc::new(Mutex::new(SpotController::new(
        controller_account,
        orderbooks,
        bank_controller.clone(),
        Arc::new(Mutex::new(EventManager::new())), // TEMPORARY
    )));

    // create orderbooks
    for i in 0..n_orderbooks {
        let creator_kp = &keypairs[0];

        // create orderbook assets
        for _ in 0..2 {
            bank_controller
                .lock()
                .unwrap()
                .create_asset(creator_kp.public())
                .unwrap();
        }

        // create orderbook
        let base_asset_id = 2 * i;
        let quote_asset_id = 2 * i + 1;
        spot_controller
            .lock()
            .unwrap()
            .create_orderbook(base_asset_id, quote_asset_id)
            .unwrap();

        for k in 1..n_users {
            let user = keypairs[k].public();

            // transfer assets from creator to user
            bank_controller
                .lock()
                .unwrap()
                .transfer(creator_kp.public(), user, base_asset_id, 10000000)
                .unwrap();
            bank_controller
                .lock()
                .unwrap()
                .transfer(creator_kp.public(), user, quote_asset_id, 1000000)
                .unwrap();

            // place orders to orderbook
            let bid_request = LimitOrderRequest::new(
                base_asset_id,
                quote_asset_id,
                OrderSide::Bid as u64,
                n_users as u64 % 100 + 1,
                1,
            );
            let ask_request = LimitOrderRequest::new(
                base_asset_id,
                quote_asset_id,
                OrderSide::Ask as u64,
                n_users as u64 % 100 + 101,
                1,
            );
            spot_controller
                .lock()
                .unwrap()
                .get_orderbook(base_asset_id, quote_asset_id)
                .unwrap()
                .place_limit_order(user, &bid_request)
                .unwrap();
            spot_controller
                .lock()
                .unwrap()
                .get_orderbook(base_asset_id, quote_asset_id)
                .unwrap()
                .place_limit_order(keypairs[k].public(), &ask_request)
                .unwrap();
        }
    }

    let spot_controller_clone = spot_controller.lock().unwrap().clone();

    // create orders on the books
    c.bench_function("catchup_spot_get_catchup_state", |b| {
        b.iter(|| spot_controller_clone.get_catchup_state())
    });

    // create orders on the books
    c.bench_function("catchup_spot_controller_clone", |b| {
        b.iter(|| spot_controller_clone.clone())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
