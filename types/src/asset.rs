//! 
//! TODO
//! 1.) Move asset addr to proper addr
//! 2.) Add asset fields
//! 
use std::fmt::Debug;
pub type AssetId = u64;
pub type AssetAddr = u64;

#[derive(Debug)]
pub struct Asset {
    pub asset_id: AssetId,
    pub asset_addr: AssetAddr,
}
