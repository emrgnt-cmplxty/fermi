//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::account::AccountPubKey;
use serde::{Deserialize, Serialize};

pub type AssetId = u64;
pub type AssetAddr = u64;
// The orderbook is looked up by AssetPairKey with format {AssetId_0}_{AssetId_1}
pub type AssetPairKey = String;

pub const PRIMARY_ASSET_ID: u64 = 0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub asset_id: AssetId,
    pub owner_pubkey: AccountPubKey,
}
