#[macro_use]
extern crate criterion;

use criterion::{Criterion, Throughput};
use gdex_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    traits::{Signature, SigningKey, Uniform},
};
use rand::{prelude::ThreadRng, thread_rng};
use types::spot::{DiemCryptoMessage};

const NUMBER_OF_MESSAGES: i32 = 1024;
const NUMBER_OF_ACCOUNTS: i32 = 1024;

// we pass in the number of messages we want to verify, a list of signatures, a list of private keys, and a list of public keys
#[inline]
fn basic_sig_verify(
    sigs: &mut Vec<Ed25519Signature>,
    public_keys: &mut Vec<Ed25519PublicKey>,
    possible_messages: &mut Vec<DiemCryptoMessage>,
) {
    let messages_per_account = NUMBER_OF_MESSAGES / NUMBER_OF_ACCOUNTS;
    for i in 0..NUMBER_OF_ACCOUNTS {
        let public_key = &public_keys[i as usize];
        for x in 0..messages_per_account {
            let sig = &sigs[((i * messages_per_account) + x) as usize];
            let msg = &possible_messages[x as usize];
            sig.verify(msg, public_key).unwrap();
        }
    }
}

#[cfg(feature = "batch")]
fn batch_sig_verify(keys_and_signatures: &Vec<(Ed25519PublicKey, Ed25519Signature)>) {
    let msg = DiemCryptoMessage("dummy".to_string());
    // gives us 1 if it was verified and 0 if it wasn't
    Signature::batch_verify(&msg, keys_and_signatures.to_vec()).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let messages_per_account = NUMBER_OF_MESSAGES / NUMBER_OF_ACCOUNTS;
    // getting list of possible raw messages
    let messages_pre_format: Vec<i32> = (0..messages_per_account).collect();
    // getting formatted list of possible messages
    let mut messages: Vec<DiemCryptoMessage> = messages_pre_format
        .iter()
        .map(|&_| DiemCryptoMessage("dummy".to_string()))
        .collect();
    // instantiating private keys
    let mut private_keys: Vec<Ed25519PrivateKey> = Vec::new();
    // instantiatng public keys
    let mut public_keys: Vec<Ed25519PublicKey> = Vec::new();
    // instantiating signatures
    let mut sigs: Vec<Ed25519Signature> = Vec::new();
    // instantiating joint vector
    let mut keys_and_signatures: Vec<(Ed25519PublicKey, Ed25519Signature)> = Vec::new();

    // looping through accounts
    for _i in 0..NUMBER_OF_ACCOUNTS {
        let mut csprng: ThreadRng = thread_rng();
        let private_key: Ed25519PrivateKey = Ed25519PrivateKey::generate(&mut csprng);
        let public_key: Ed25519PublicKey = (&private_key).into();
        // pushing transactions
        for _ in 0..messages_per_account {
            let msg: DiemCryptoMessage = DiemCryptoMessage("dummy".to_string());
            let sig: Ed25519Signature = private_key.sign(&msg);
            sigs.push(sig.clone());
            keys_and_signatures.push((public_key, sig));
        }
        // adding keys to vector
        private_keys.push(private_key);
        public_keys.push(public_key);
    }
    let mut group = c.benchmark_group("sig_verification");
    group.throughput(Throughput::Elements((NUMBER_OF_MESSAGES) as u64));
    group.bench_function("basic_sig_verify", |b| {
        b.iter(|| basic_sig_verify(&mut sigs, &mut public_keys, &mut messages))
    });
    #[cfg(feature = "batch")]
    group.bench_function("batch_sig_verify", |b| {
        b.iter(|| batch_sig_verify(&keys_and_signatures))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
