// crate
use crate::utils::engine::order_book::{OrderId, Orderbook};

// gdex
use gdex_types::{
    account::AccountPubKey,
    asset::AssetId,
    transaction::{FuturesOrder, FuturesPosition},
};

// external
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

// TODO - move futures .proto to this folder

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CondensedOrder {
    pub side: u64,
    pub quantity: u64,
    pub price: u64,
    pub base_asset_id: u64,
}

impl CondensedOrder {
    pub(crate) fn from_order(order: &FuturesOrder, base_asset_id: u64) -> Self {
        Self {
            side: order.side,
            quantity: order.quantity,
            price: order.price,
            base_asset_id,
        }
    }
}

/// SpotOrderAccount is consumed by the SpotController
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FuturesAccount {
    pub open_orders: Vec<FuturesOrder>,
    pub position: Option<FuturesPosition>,
}
impl FuturesAccount {
    pub fn new() -> Self {
        FuturesAccount {
            position: None,
            open_orders: Vec::new(),
        }
    }
}

impl Default for FuturesAccount {
    fn default() -> Self {
        Self::new()
    }
}

pub type AssetPrice = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FuturesMarket {
    pub max_leverage: u64,
    // TODO - it is gross to have to store the base and quote asset id here
    // we should consider removing these from the orderbook
    pub base_asset_id: u64,
    pub quote_asset_id: u64,
    pub latest_price: AssetPrice,
    pub order_to_account: HashMap<OrderId, AccountPubKey>,
    pub accounts: HashMap<AccountPubKey, FuturesAccount>,
    pub orderbook: Orderbook,
    // reference to parent Marketplace deposits
    pub marketplace_deposits: Weak<Mutex<HashMap<AccountPubKey, i64>>>,
    pub liquidation_fee_percent: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Marketplace {
    pub quote_asset_id: u64,
    pub latest_time: u64,
    pub markets: HashMap<AssetId, FuturesMarket>,
    // i64 is necessary because deposits can go negative given inadequate liquidations
    // Arc + Mutex wrapper is necessary as a reference to deposits must be passed to each FuturesMarket
    pub deposits: Arc<Mutex<HashMap<AccountPubKey, i64>>>,
}

pub type AccountState = Vec<(Vec<FuturesOrder>, Option<FuturesPosition>)>;
