//! Copyright (c) 2018 Anton Dort-Golts
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

use gdex_types::{
    asset::AssetId,
    order_book::{
        OrderSide,
        OrderRequest
    }
};
use std::time::SystemTime;

/* Constructors */

/// Create request for the new market order
pub fn create_market_order_request(
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    side: OrderSide,
    quantity: u64,
    local_timestamp: SystemTime,
) -> OrderRequest {
    OrderRequest::Market {
        base_asset_id,
        quote_asset_id,
        quantity,
        side,
        local_timestamp,
    }
}

/// Create request for the new limit order
pub fn create_limit_order_request(
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    side: OrderSide,
    price: u64,
    quantity: u64,
    local_timestamp: SystemTime,
) -> OrderRequest {
    OrderRequest::Limit {
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        local_timestamp,
    }
}

/// Create request for changing price/quantity for the active limit order.
///
/// Note: do not change order side!
/// Instead cancel existing order and create a new one.
pub fn create_update_order_request(
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    order_id: u64,
    side: OrderSide,
    price: u64,
    quantity: u64,
    local_timestamp: SystemTime,
) -> OrderRequest {
    OrderRequest::Update {
        base_asset_id,
        quote_asset_id,
        order_id,
        side,
        price,
        quantity,
        local_timestamp,
    }
}

/// Create request for cancelling active limit order
pub fn create_cancel_order_request(
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    order_id: u64,
    side: OrderSide,
    local_timestamp: SystemTime,
) -> OrderRequest {
    OrderRequest::Cancel {
        base_asset_id,
        quote_asset_id,
        order_id,
        side,
        local_timestamp,
    }
}
