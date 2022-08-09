//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::asset::AssetId;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::SystemTime};

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub enum OrderSide {
    Bid,
    Ask,
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

#[derive(Debug)]
pub enum Success {
    Accepted {
        order_id: u64,
        order_type: OrderType,
        ts: SystemTime,
    },

    Filled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: u64,
        quantity: u64,
        ts: SystemTime,
    },

    PartiallyFilled {
        order_id: u64,
        side: OrderSide,
        order_type: OrderType,
        price: u64,
        quantity: u64,
        ts: SystemTime,
    },

    Amended {
        order_id: u64,
        price: u64,
        quantity: u64,
        ts: SystemTime,
    },

    Cancelled {
        order_id: u64,
        ts: SystemTime,
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