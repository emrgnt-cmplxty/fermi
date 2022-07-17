use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::fmt::Debug;

use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use types::{
    asset::{AssetId},
    orderbook::{OrderSide}
};

#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum OrderRequest
{
    NewMarketOrder {
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        qty: u64,
        ts: SystemTime,
    },

    NewLimitOrder {
        base_asset: AssetId,
        quote_asset: AssetId,
        side: OrderSide,
        price: u64,
        qty: u64,
        ts: SystemTime,
    },

    AmendOrder {
        id: u64,
        side: OrderSide,
        price: u64,
        qty: u64,
        ts: SystemTime,
    },

    CancelOrder {
        id: u64,
        side: OrderSide,
        //ts: SystemTime,
    },
}


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

    OrderRequest::NewMarketOrder {
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

    OrderRequest::NewLimitOrder {
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

    OrderRequest::AmendOrder {
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
