use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};

pub type OrderId = u64;

#[derive(Debug, BCSCryptoHash, CryptoHasher, Serialize, Deserialize)]
pub struct DiemCryptoMessage(pub String);
