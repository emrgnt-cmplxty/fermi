// Copyright (c) 2022, BTI
// SPDX-License-Identifier: Apache-2.0
use narwhal_crypto::ed25519::{
    Ed25519KeyPair, Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature,
};

pub type AccountPubKey = Ed25519PublicKey;
pub type AccountPrivKey = Ed25519PrivateKey;
pub type AccountSignature = Ed25519Signature;
pub type AccountKeyPair = Ed25519KeyPair;
pub type AccountBalance = u64;

#[cfg(any(test, feature = "testing"))]
pub mod account_test_functions {
    use super::*;
    use narwhal_crypto::traits::KeyPair;
    use rand::{rngs::StdRng, SeedableRng};

    pub fn generate_keypair_vec(seed: [u8; 32]) -> Vec<AccountKeyPair> {
        let mut rng = StdRng::from_seed(seed);
        (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
    }
}
