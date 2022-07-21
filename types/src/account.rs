use gdex_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};

pub type AccountPubKey = Ed25519PublicKey;
pub type AccountPrivKey = Ed25519PrivateKey;
pub type AccountSignature = Ed25519Signature;
pub type AccountBalance = u64;
