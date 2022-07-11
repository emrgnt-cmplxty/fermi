
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct Order<Asset>
where
    Asset: Debug + Clone,
{
    pub order_id: u64,
    pub base_asset: Asset,
    pub quote_asset: Asset,
    pub side: OrderSide,
    pub price: u64,
    pub qty: u64,
}


#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum OrderType {
    Market,
    Limit,
}
