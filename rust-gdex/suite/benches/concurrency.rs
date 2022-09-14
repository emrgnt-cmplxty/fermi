// Copyright (c) 2022, BTI
// SPDX-License-Identifier: Apache-2.0
// to run this code, run cargo bench mutex_lock, for ex.
// TODO - cleanup this benchmark file

extern crate criterion;

use criterion::*;

use gdex_controller::bank::BankController;
use std::sync::{Arc, Mutex};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{channel, Receiver, Sender},
};

fn criterion_benchmark(c: &mut Criterion) {
    // test mutex
    fn obtain_mutex_lock(bank_controller: &Mutex<BankController>) {
        let _ = bank_controller.lock().unwrap();
    }
    let bank_controller = Mutex::new(BankController::default());

    c.bench_function("mutex_lock", move |b| b.iter(|| obtain_mutex_lock(&bank_controller)));

    // test arc mutex
    fn obtain_arc_mutex_lock(bank_controller: &Arc<Mutex<BankController>>) {
        let _ = bank_controller.lock().unwrap();
    }
    let bank_controller = Arc::new(Mutex::new(BankController::default()));

    c.bench_function("arc_mutex_lock", move |b| {
        b.iter(|| obtain_arc_mutex_lock(&bank_controller))
    });

    // test channels by checking the speed to send 1_000 messages
    pub const DEFAULT_CHANNEL_SIZE: usize = 1_000;

    async fn init_channel_64(_bytes_sent: [u8; 64]) {
        let (_tx, mut _rx): (Sender<[u8; 64]>, Receiver<[u8; 64]>) = channel(DEFAULT_CHANNEL_SIZE);
    }

    async fn init_channel_512(_bytes_sent: [u8; 512]) {
        let (_tx, mut _rx): (Sender<[u8; 512]>, Receiver<[u8; 512]>) = channel(DEFAULT_CHANNEL_SIZE);
    }

    async fn send_channel_64_1_000(bytes_sent: [u8; 64]) {
        let (tx, mut _rx) = channel(DEFAULT_CHANNEL_SIZE);
        let mut i = 0;
        while i < 1_000 {
            tx.send(bytes_sent).await.unwrap();
            i += 1;
        }
    }

    async fn send_channel_512_1_000(bytes_sent: [u8; 512]) {
        let (tx, mut _rx) = channel(DEFAULT_CHANNEL_SIZE);
        let mut i = 0;
        while i < 1_000 {
            tx.send(bytes_sent).await.unwrap();
            i += 1;
        }
    }

    async fn send_and_receive_channel_64_1_000(bytes_sent: [u8; 64]) {
        let (tx, mut rx) = channel(DEFAULT_CHANNEL_SIZE);
        let mut i = 0;
        while i < 1_000 {
            tx.send(bytes_sent).await.unwrap();
            let _ = rx.recv().await.unwrap();
            i += 1;
        }
    }

    async fn send_and_receive_channel_512_1_000(bytes_sent: [u8; 512]) {
        let (tx, mut rx) = channel(DEFAULT_CHANNEL_SIZE);
        let mut i = 0;
        while i < 1_000 {
            tx.send(bytes_sent).await.unwrap();
            let _ = rx.recv().await.unwrap();
            i += 1;
        }
    }

    c.bench_function("concurrency_init_channel_64", move |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| init_channel_64(black_box([0_u8; 64])))
    });

    c.bench_function("concurrency_send_channel_64_1_000", move |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| send_channel_64_1_000(black_box([0_u8; 64])))
    });

    c.bench_function("concurrency_send_and_receive_channel_64_1_000", move |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| send_and_receive_channel_64_1_000(black_box([0_u8; 64])))
    });

    c.bench_function("concurrency_init_channel_512_1_000", move |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| init_channel_512(black_box([0_u8; 512])))
    });

    c.bench_function("concurrency_send_channel_512_1_000", move |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| send_channel_512_1_000(black_box([0_u8; 512])))
    });

    c.bench_function("concurrency_send_and_receive_channel_512_1_000", move |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| send_and_receive_channel_512_1_000(black_box([0_u8; 512])))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
