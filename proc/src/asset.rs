//! 
//! TODO
//! 1.) Move asset addr to proper addr
//! 2.) Add asset fields
//! 
extern crate engine;

use std::fmt::Debug;

pub type AssetId = u64;
pub type AssetAddr = u64;

// add fields to asset 
#[derive(Debug)]
pub struct Asset {
    pub asset_id: AssetId,
    pub asset_addr: AssetAddr,
}