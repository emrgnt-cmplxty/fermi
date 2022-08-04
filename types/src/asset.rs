//!
//! TODO
//! 1.) Move asset addr to proper addr
//! 2.) Add asset fields
//!
use super::account::AccountPubKey;

pub type AssetId = u64;
pub type AssetAddr = u64;
// orderbook is looked up by AssetPairKey with format {AssetId_0}_{AssetId_1}
pub type AssetPairKey = String;
pub struct Asset {
    pub asset_id: AssetId,
    pub owner_pubkey: AccountPubKey,
}