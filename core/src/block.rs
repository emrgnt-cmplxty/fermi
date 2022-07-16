
// use super::transaction;

use std::fmt::Debug;
use diem_crypto::{
    hash::HashValue,
};
// use transaction::{Transaction};


pub struct BlockContainer<Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub blocks: Vec<Block<Asset>>,
}

pub struct Block<Asset> 
where
    Asset: Debug + Clone + Copy + Eq, 
{
    pub txns: Vec<Transaction<Asset>>,
    pub hash: HashValue
}