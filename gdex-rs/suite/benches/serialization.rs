// gdex
use gdex_controller::bank::proto::create_payment_transaction;
use gdex_types::{
    account::AccountKeyPair,
    crypto::KeypairTraits,
    error::GDEXError,
    transaction::{deserialize_protobuf, serialize_protobuf, SignedTransaction},
};
// narwhal
use fastcrypto::DIGEST_LEN;
use narwhal_types::{Batch, CertificateDigest, WorkerMessage};
// external
use criterion::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn keys(seed: [u8; 32]) -> Vec<AccountKeyPair> {
    let mut rng = StdRng::from_seed(seed);
    (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
}

fn verify_incoming_transaction(signed_transaction_bytes: Vec<u8>) -> Result<(), GDEXError> {
    match bincode::deserialize::<SignedTransaction>(&signed_transaction_bytes) {
        Ok(signed_transaction) => match signed_transaction.verify_signature() {
            Ok(_) => Ok(()),
            Err(sig_error) => Err(sig_error),
        },
        // deserialization failed
        Err(_) => Err(GDEXError::DeserializationError),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    fn get_signed_transaction(sender_seed: [u8; 32], receiver_seed: [u8; 32], amount: u64) -> SignedTransaction {
        let kp_sender = keys(sender_seed).pop().unwrap();
        let kp_receiver = keys(receiver_seed).pop().unwrap();
        let certificate_digest = CertificateDigest::new([0; DIGEST_LEN]);
        let transaction =
            create_payment_transaction(kp_sender.public(), certificate_digest, kp_receiver.public(), 0, amount);
        transaction.sign(&kp_sender).unwrap()
    }

    // bench serializing singletons
    fn serialize_1_000(sender_seed: [u8; 32], receiver_seed: [u8; 32]) {
        let signed_transaction = get_signed_transaction(sender_seed, receiver_seed, 10);

        let mut i = 0;
        while i < 1_000 {
            // wrap signed transaction in black box to protect compiler from advance knowledge
            let _ = black_box(bincode::serialize(&signed_transaction).unwrap());
            i += 1;
        }
    }

    // bench deserializing singletons
    fn deserialize_1_000(sender_seed: [u8; 32], receiver_seed: [u8; 32]) {
        let signed_transaction = get_signed_transaction(sender_seed, receiver_seed, 10);
        let signed_transaction_serialized = bincode::serialize(&signed_transaction).unwrap();

        let mut i = 0;
        while i < 1_000 {
            // wrap signed transaction in black box to protect compiler from advance knowledge
            let _: SignedTransaction = black_box(bincode::deserialize(&signed_transaction_serialized).unwrap());
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
        let signed_transaction = get_signed_transaction([0; 32], [1; 32], amount);
        batch.push(bincode::serialize(&signed_transaction).unwrap());
        i += 1;
    }

    // bench deserializing a batch w/ no verification
    fn deserialize_batch_method1(batch: &[u8]) {
        match bincode::deserialize(batch).unwrap() {
            WorkerMessage::Batch(Batch(signed_transactions)) => {
                for raw_signed_transaction in signed_transactions {
                    let signed_transaction_bytes: Vec<u8> = raw_signed_transaction.to_vec();

                    let _: SignedTransaction = bincode::deserialize(&signed_transaction_bytes).unwrap();
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
            WorkerMessage::Batch(Batch(signed_transactions)) => {
                for raw_signed_transaction in signed_transactions {
                    let signed_transaction_bytes: Vec<u8> = raw_signed_transaction.to_vec();

                    verify_incoming_transaction(signed_transaction_bytes).unwrap();
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

    // protobuf
    fn protobuf_serialize_1_000(sender_seed: [u8; 32], receiver_seed: [u8; 32]) {
        let signed_transaction = get_signed_transaction(sender_seed, receiver_seed, 10);

        let mut i = 0;
        while i < 1_000 {
            let _ = serialize_protobuf(&signed_transaction);
            i += 1;
        }
    }

    fn protobuf_deserialize_1_000(sender_seed: [u8; 32], receiver_seed: [u8; 32]) {
        let signed_transaction_serialized = serialize_protobuf(&get_signed_transaction(sender_seed, receiver_seed, 10));

        let mut i = 0;
        while i < 1_000 {
            let signed_transaction: SignedTransaction = deserialize_protobuf(&signed_transaction_serialized).unwrap();
            i += 1;
        }
    }

    c.bench_function("serialization_protobuf_serialize_1_000", move |b| {
        b.iter(|| protobuf_serialize_1_000(black_box([0_u8; 32]), black_box([1_u8; 32])))
    });

    c.bench_function("serialization_protobuf_deserialize_1_000", move |b| {
        b.iter(|| protobuf_deserialize_1_000(black_box([0_u8; 32]), black_box([1_u8; 32])))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
