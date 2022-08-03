use std::time::SystemTime;
use types::{AssetId, OrderRequest, OrderSide};

/* Constructors */

/// Create request for the new market order
pub fn new_market_order_request(
    base_asset: AssetId,
    quote_asset: AssetId,
    side: OrderSide,
    quantity: u64,
    ts: SystemTime,
) -> OrderRequest {
    OrderRequest::Market {
        base_asset,
        quote_asset,
        quantity,
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
    quantity: u64,
    ts: SystemTime,
) -> OrderRequest {
    OrderRequest::Limit {
        base_asset,
        quote_asset,
        side,
        price,
        quantity,
        ts,
    }
}

/// Create request for changing price/quantity for the active limit order.
///
/// Note: do not change order side!
/// Instead cancel existing order and create a new one.
pub fn amend_order_request(id: u64, side: OrderSide, price: u64, quantity: u64, ts: SystemTime) -> OrderRequest {
    OrderRequest::Amend {
        id,
        side,
        price,
        quantity,
        ts,
    }
}

/// Create request for cancelling active limit order
pub fn limit_order_cancel_request(order_id: u64, side: OrderSide) -> OrderRequest {
    OrderRequest::CancelOrder { id: order_id, side }
}
