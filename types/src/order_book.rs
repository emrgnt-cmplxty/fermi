//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::asset::AssetId;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::SystemTime};

pub type OrderId = u64;

#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Debug)]
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Depth {
    pub price: u64,
    pub quantity: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct OrderbookSnap {
    pub bids: Vec<Depth>,
    pub asks: Vec<Depth>,
}