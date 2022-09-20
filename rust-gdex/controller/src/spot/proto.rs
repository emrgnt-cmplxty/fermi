// IMPORTS

// gdex
use gdex_types::{
    account::{
        AccountPubKey
    },
    transaction::{
        Transaction, ControllerType, RequestType, serialize_protobuf, create_transaction
    },
};

// mysten
use narwhal_types::{CertificateDigest};

// MODULE IMPORTS

#[path = "./generated/spot_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod spot_requests;

pub use spot_requests::*;

// REQUEST BUILDERS

// SPOT REQUESTS

pub fn create_create_orderbook_request(base_asset_id: u64, quote_asset_id: u64) -> CreateOrderbookRequest {
    CreateOrderbookRequest {
        base_asset_id,
        quote_asset_id,
    }
}

pub fn create_market_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    quantity: u64,
) -> MarketOrderRequest {
    MarketOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        quantity,
    }
}

pub fn create_limit_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
) -> LimitOrderRequest {
    LimitOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
    }
}

pub fn create_update_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    order_id: u64,
) -> UpdateOrderRequest {
    UpdateOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        price,
        quantity,
        order_id,
    }
}

pub fn create_cancel_order_request(
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    order_id: u64,
) -> CancelOrderRequest {
    CancelOrderRequest {
        base_asset_id,
        quote_asset_id,
        side,
        order_id,
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
    let request = create_create_orderbook_request(base_asset_id, quote_asset_id);

    create_transaction(
        sender,
        ControllerType::Spot,
        RequestType::CreateOrderbook,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
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
    let request = create_market_order_request(base_asset_id, quote_asset_id, side, quantity);

    create_transaction(
        sender,
        ControllerType::Spot,
        RequestType::MarketOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
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
    let request = create_limit_order_request(base_asset_id, quote_asset_id, side, price, quantity);

    create_transaction(
        sender,
        ControllerType::Spot,
        RequestType::LimitOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
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
    let request = create_update_order_request(base_asset_id, quote_asset_id, side, price, quantity, order_id);

    create_transaction(
        sender,
        ControllerType::Spot,
        RequestType::UpdateOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
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
    let request = create_cancel_order_request(base_asset_id, quote_asset_id, side, order_id);

    create_transaction(
        sender,
        ControllerType::Spot,
        RequestType::CancelOrder,
        recent_block_hash,
        fee,
        serialize_protobuf(&request),
    )
}