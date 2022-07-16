
use std::fmt::Debug;
use diem_crypto::{
    hash::HashValue,
};

use super::transaction::{Transaction};


pub struct BlockContainer
{
    pub blocks: Vec<Block>,
}

pub struct Block
{
    pub txns: Vec<Transaction>,
    pub hash: HashValue
}