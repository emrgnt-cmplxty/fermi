use diem_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519PrivateKey, Ed25519Signature},
};

pub type AccountPubKey = Ed25519PublicKey;
pub type AccountPrivKey = Ed25519PrivateKey;
pub type AccountSignature = Ed25519Signature;
pub type AccountBalance = u64;

#[derive(Debug)]
pub enum AccountError {
    Creation(String),
    Lookup(String),
    OrderProc(String),
    Payment(String),
    Vote(String),
    Signing(String),
}
