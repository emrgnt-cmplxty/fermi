// Copyright (c) 2022, BTI
// SPDX-License-Identifier: Apache-2.0
// to run this code, run cargo bench mutex_lock, for ex.
// TODO - cleanup this benchmark file

extern crate bincode;
extern crate criterion;

use criterion::*;
use gdex_types::{
    account::{AccountKeyPair, AccountSignature},
    crypto::{KeypairTraits, Signer},
    error::GDEXError,
    new_transaction::{ConsensusTransaction, NewSignedTransaction, NewTransaction, new_create_payment_transaction, sign_transaction}
};
use narwhal_crypto::{Hash, DIGEST_LEN};
use narwhal_types::{Batch, CertificateDigest, WorkerMessage};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn keys(seed: [u8; 32]) -> Vec<AccountKeyPair> {
    let mut rng = StdRng::from_seed(seed);
    (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
}

fn verify_incoming_transaction(raw_consensus_transaction: Vec<u8>) -> Result<(), GDEXError> {
    // remove trailing zeros & deserialize transaction
    let consensus_transaction_result = ConsensusTransaction::deserialize(raw_consensus_transaction);

    match consensus_transaction_result {
        Ok(consensus_transaction) => {
            match consensus_transaction.get_payload() {
                Ok(signed_transaction) => {
                    match verify_signature(sign_transaction) {
                        Ok(_) => {
                            Ok(())
                        }
                        Err(sig_error) => Err(sig_error),
                    }
                },
                Err(get_payload_error) => Err(get_payload_error),
            }
        }
        // deserialization failed
        Err(derserialize_err) => Err(derserialize_err),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    fn get_consensus_transaction(sender_seed: [u8; 32], receiver_seed: [u8; 32], amount: u64) -> ConsensusTransaction {
        let kp_sender = keys(sender_seed).pop().unwrap();
        let kp_receiver = keys(receiver_seed).pop().unwrap();
        let certificate_digest = CertificateDigest::new([0; DIGEST_LEN]);
        let gas: u64 = 1000;
        let new_transaction = new_create_payment_transaction(kp_sender.public().clone(), kp_receiver.public(), 0, amount, gas, certificate_digest);
        let new_signed_transaction = match sign_transaction(kp_sender, new_transaction) {
            Ok(t) => t,
            _ => panic!("Error signing transaction"),
        };

        ConsensusTransaction::new(new_signed_transaction)
    }

    // bench serializing singletons
    fn serialize_1_000(sender_seed: [u8; 32], receiver_seed: [u8; 32]) {
        let consensus_transaction = get_consensus_transaction(sender_seed, receiver_seed, 10);

        let mut i = 0;
        while i < 1_000 {
            // wrap signed transaction in black box to protect compiler from advance knowledge
            let _ = black_box(consensus_transaction.clone()).serialize().unwrap();
            i += 1;
        }
    }

    // bench deserializing singletons
    fn deserialize_1_000(sender_seed: [u8; 32], receiver_seed: [u8; 32]) {
        let consensus_transaction_serialized = get_consensus_transaction(sender_seed, receiver_seed, 10)
            .serialize()
            .unwrap();

        let mut i = 0;
        while i < 1_000 {
            // wrap signed transaction in black box to protect compiler from advance knowledge
            let _ = ConsensusTransaction::deserialize(black_box(consensus_transaction_serialized.clone())).unwrap();
            i += 1;
        }
    }

    c.bench_function("serialization_serialize_1_000", move |b| {
        b.iter(|| serialize_1_000(black_box([0_u8; 32]), black_box([1_u8; 32])))
    });

    c.bench_function("serialization_deserialize_1_000", move |b| {
        b.iter(|| deserialize_1_000(black_box([0_u8; 32]), black_box([1_u8; 32])))
    });

    let mut i = 0;
    let mut batch = Vec::new();
    while i < 1_000 {
        let amount = rand::thread_rng().gen_range(10..100);
        let consensus_transaction = get_consensus_transaction([0; 32], [1; 32], amount);
        batch.push(bincode::serialize(&consensus_transaction).unwrap());
        i += 1;
    }

    // bench deserializing a batch w/ no verification
    fn deserialize_batch_method1(batch: &[u8]) {
        match bincode::deserialize(batch).unwrap() {
            WorkerMessage::Batch(Batch(transactions)) => {
                for raw_transaction in transactions {
                    let transaction: Vec<u8> = raw_transaction
                        .to_vec()
                        .collect();

                    let _ = ConsensusTransaction::deserialize(transaction).unwrap();
                }
            }
            _ => {
                panic!("error occurred in deserialize_batch_method1 while deserializing")
            }
        };
    }

    // bench deserializing a batch w/ verification
    fn deserialize_batch_and_verify_method1(batch: &[u8]) {
        match bincode::deserialize(batch).unwrap() {
            WorkerMessage::Batch(Batch(transactions)) => {
                for raw_transaction in transactions {
                    let transaction: Vec<u8> = raw_transaction
                        .to_vec()
                        .collect();

                    verify_incoming_transaction(transaction).unwrap();
                }
            }
            _ => {
                panic!("error occurred in deserialize_batch_and_verify_method1 while deserializing")
            }
        };
    }

    let message = WorkerMessage::Batch(Batch(batch.clone()));
    let serialized = bincode::serialize(&message).unwrap();

    c.bench_function("serialization_deserialize_batch_method1_1_000", move |b| {
        b.iter(|| deserialize_batch_method1(black_box(&serialized[..])))
    });

    let message = WorkerMessage::Batch(Batch(batch));
    let serialized = bincode::serialize(&message).unwrap();

    c.bench_function("serialization_deserialize_batch_and_verify_method1_1_000", move |b| {
        b.iter(|| deserialize_batch_and_verify_method1(black_box(&serialized[..])))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
