extern crate core;
use types::{asset::AssetId, orderbook::OrderSide};
use core::transaction::OrderRequest;
use std::time::SystemTime;

/* Constructors */

/// Create request for the new market order
pub fn new_market_order_request(
    base_asset: AssetId,
    quote_asset: AssetId,
    side: OrderSide,
    qty: u64,
    ts: SystemTime,
) -> OrderRequest
{
    OrderRequest::Market {
        base_asset,
        quote_asset,
        qty,
        side,
        ts,
    }
}


/// Create request for the new limit order
pub fn new_limit_order_request(
    base_asset: AssetId,
    quote_asset: AssetId,
    side: OrderSide,
    price: u64,
    qty: u64,
    ts: SystemTime,
) -> OrderRequest
{

    OrderRequest::Limit {
        base_asset,
        quote_asset,
        side,
        price,
        qty,
        ts,
    }
}


/// Create request for changing price/qty for the active limit order.
///
/// Note: do not change order side!
/// Instead cancel existing order and create a new one.
pub fn amend_order_request(
    id: u64,
    side: OrderSide,
    price: u64,
    qty: u64,
    ts: SystemTime,
) -> OrderRequest
{

    OrderRequest::Amend {
        id,
        side,
        price,
        qty,
        ts,
    }
}


/// Create request for cancelling active limit order
pub fn limit_order_cancel_request(order_id: u64, side: OrderSide) -> OrderRequest
{
    OrderRequest::CancelOrder { id: order_id, side }
}
