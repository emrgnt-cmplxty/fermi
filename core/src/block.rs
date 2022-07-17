use std::{fmt::Debug};

use super::transaction::{TxnRequest, TxnVariant};
use diem_crypto::{
    hash::{CryptoHash, HashValue},
};
use types::{
    spot::{TestDiemCrypto}
};

pub struct BlockContainer <Variant>
where
    Variant: Debug + Clone + CryptoHash,
{
    pub blocks: Vec<Block<Variant>>,
}

pub struct Block <Variant>
where
    Variant: Debug + Clone + CryptoHash,
{
    pub txns: Vec<TxnRequest<Variant>>,
    pub block_hash: HashValue
}

pub fn generate_block_hash(txns: &Vec<TxnRequest<TxnVariant>>) -> HashValue{
    let mut hash_string = String::from("");
    for txn in txns {
        hash_string += &txn.txn.hash().to_string();
    }
    return TestDiemCrypto(hash_string).hash()
}
