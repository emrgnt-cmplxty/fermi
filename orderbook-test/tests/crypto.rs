// pub use crypto::ed25519

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

fn verify(c: &mut Criterion) {
    let mut csprng: ThreadRng = thread_rng();
    let priv_key = Ed25519PrivateKey::generate(&mut csprng);
    let pub_key: Ed25519PublicKey = (&priv_key).into();
    let msg = TestDiemCrypto("".to_string());
    let sig: Ed25519Signature = priv_key.sign(&msg);

    c.bench_function("Ed25519 signature verification", move |b| {
        b.iter(|| sig.verify(&msg, &pub_key))
    });
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn test_testing(){
        assert_eq!(1, 1);
    }
    // have a test where 
}
