use std::{fmt::Debug};
use diem_crypto::{
    hash::{CryptoHash, HashValue},
};

use super::transaction::{TxnRequest};

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