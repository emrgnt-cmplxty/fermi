// fermi
use crate::account::AccountPubKey;
// external
use serde::{Deserialize, Serialize};

pub type AssetId = u64;
pub type AssetAddr = u64;
pub type AssetPrice = u64;
pub type AssetAmount = u64;

// The orderbook is looked up by AssetPairKey with format {AssetId_0}_{AssetId_1}
pub type AssetPairKey = String;
pub type FuturesOrderbookKey = String;

pub const PRIMARY_ASSET_ID: u64 = 0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub asset_id: AssetId,
    pub owner_pubkey: AccountPubKey,
}