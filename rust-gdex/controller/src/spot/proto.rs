// IMPORTS

// gdex
use gdex_types::{
    account::AccountPubKey,
    transaction::{serialize_protobuf, ControllerType, RequestType, Transaction},
};

// mysten
use narwhal_types::CertificateDigest;

// MODULE IMPORTS

#[path = "./generated/spot_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod spot_requests;

pub use spot_requests::*;

// INTERFACE

impl CreateOrderbookRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64) -> Self {
        CreateOrderbookRequest {
            base_asset_id,
            quote_asset_id,
        }
    }
}

impl MarketOrderRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64, side: u64, quantity: u64) -> Self {
        MarketOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            quantity,
        }
    }
}

impl LimitOrderRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64, side: u64, price: u64, quantity: u64) -> Self {
        LimitOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
        }
    }
}

impl UpdateOrderRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64, side: u64, price: u64, quantity: u64, order_id: u64) -> Self {
        UpdateOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
            order_id,
        }
    }
}

impl CancelOrderRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64, side: u64, order_id: u64) -> Self {
        CancelOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            order_id,
        }
    }
}

// TRANSACTION BUILDERS

pub fn create_create_orderbook_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    Transaction::new(
        &sender,
        ControllerType::Spot,
        RequestType::CreateOrderbook,
        recent_block_hash,
        fee,
        serialize_protobuf(&CreateOrderbookRequest::new(base_asset_id, quote_asset_id)),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_market_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    quantity: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    Transaction::new(
        &sender,
        ControllerType::Spot,
        RequestType::MarketOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&MarketOrderRequest::new(base_asset_id, quote_asset_id, side, quantity)),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_limit_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    Transaction::new(
        &sender,
        ControllerType::Spot,
        RequestType::LimitOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&LimitOrderRequest::new(
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
        )),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_update_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    order_id: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    Transaction::new(
        &sender,
        ControllerType::Spot,
        RequestType::UpdateOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&UpdateOrderRequest::new(
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
            order_id,
        )),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_cancel_order_transaction(
    sender: AccountPubKey,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    order_id: u64,
    fee: u64,
    recent_block_hash: CertificateDigest,
) -> Transaction {
    Transaction::new(
        &sender,
        ControllerType::Spot,
        RequestType::CancelOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&CancelOrderRequest::new(base_asset_id, quote_asset_id, side, order_id)),
    )
}
