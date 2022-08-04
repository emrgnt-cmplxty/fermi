//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::account::AccountPubKey;

pub type AssetId = u64;
pub type AssetAddr = u64;
// The orderbook is looked up by AssetPairKey with format {AssetId_0}_{AssetId_1}
pub type AssetPairKey = String;

pub struct Asset {
    pub asset_id: AssetId,
    pub owner_pubkey: AccountPubKey,
}