// gdex
use crate::asset::{AssetAmount, AssetId, AssetPrice};
// external
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::SystemTime};

pub type OrderId = u64;

#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Debug)]
#[repr(u64)]
pub enum OrderSide {
    Bid = 1,
    Ask = 2,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub order_id: u64,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u64,
}

impl Order {
    pub fn get_order_id(&self) -> u64 {
        self.order_id
    }

    pub fn get_base_asset(&self) -> AssetId {
        self.base_asset
    }

    pub fn get_quote_asset(&self) -> AssetId {
        self.quote_asset
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn get_price(&self) -> u64 {
        self.price
    }

    pub fn get_quantity(&self) -> u64 {
        self.quantity
    }
}

#[derive(Debug)]
pub enum OrderRequest {
    Market {
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        side: OrderSide,
        quantity: AssetAmount,
        local_timestamp: SystemTime,
    },
    Limit {
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        side: OrderSide,
        price: AssetPrice,
        quantity: AssetAmount,
        local_timestamp: SystemTime,
    },
    Update {
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        order_id: OrderId,
        side: OrderSide,
        price: AssetPrice,
        quantity: AssetAmount,
        local_timestamp: SystemTime,
    },
    Cancel {
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        order_id: OrderId,
        side: OrderSide,
        local_timestamp: SystemTime,
    },
}

#[derive(Debug)]
pub enum Success {
    Accepted {
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
        order_type: OrderType,
        timestamp: SystemTime,
    },
    Filled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    },
    PartiallyFilled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    },
    Updated {
        order_id: u64,
        side: OrderSide,
        previous_price: u64,
        previous_quantity: u64,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    },
    Cancelled {
        order_id: u64,
        side: OrderSide,
        price: u64,
        quantity: u64,
        timestamp: SystemTime,
    },
}

#[derive(Debug)]
pub enum Failed {
    Validation(String),
    DuplicateOrderID(u64),
    NoMatch(u64),
    OrderNotFound(u64),
}

pub type OrderProcessingResult = Vec<Result<Success, Failed>>;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Depth {
    pub price: u64,
    pub quantity: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OrderbookDepth {
    pub bids: Vec<Depth>,
    pub asks: Vec<Depth>,
}
