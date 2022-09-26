// IMPORTS
// crate
use crate::router::ControllerType;

// gdex
use gdex_types::{
    account::AccountPubKey,
    error::GDEXError,
    transaction::{Event, EventTypeEnum, Request, RequestTypeEnum, Transaction},
};

// mysten
use narwhal_types::CertificateDigest;

// external
use prost::bytes::Bytes;

// MODULE IMPORTS

#[path = "./generated/spot_proto.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod spot_proto;

pub use spot_proto::*;

// ENUMS

impl RequestTypeEnum for SpotRequestType {
    fn request_type_from_i32(value: i32) -> Result<Self, GDEXError> {
        match value {
            0 => Ok(SpotRequestType::CreateOrderbook),
            1 => Ok(SpotRequestType::MarketOrder),
            2 => Ok(SpotRequestType::LimitOrder),
            3 => Ok(SpotRequestType::UpdateOrder),
            4 => Ok(SpotRequestType::CancelOrder),
            _ => Err(GDEXError::DeserializationError),
        }
    }
}

impl EventTypeEnum for SpotEventType {
    fn event_type_from_i32(value: i32) -> Result<Self, GDEXError> {
        match value {
            0 => Ok(SpotEventType::OrderNew),
            1 => Ok(SpotEventType::OrderFill),
            2 => Ok(SpotEventType::OrderPartialFill),
            3 => Ok(SpotEventType::OrderUpdate),
            4 => Ok(SpotEventType::OrderCancel),
            _ => Err(GDEXError::DeserializationError),
        }
    }
}

// INTERFACE

// create orderbook

impl CreateOrderbookRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64) -> Self {
        CreateOrderbookRequest {
            base_asset_id,
            quote_asset_id,
        }
    }
}

impl Request for CreateOrderbookRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_request_type_id() -> i32 {
        SpotRequestType::CreateOrderbook as i32
    }
}

// market order

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

impl Request for MarketOrderRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_request_type_id() -> i32 {
        SpotRequestType::MarketOrder as i32
    }
}

// limit order

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

impl Request for LimitOrderRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_request_type_id() -> i32 {
        SpotRequestType::LimitOrder as i32
    }
}

// update order

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

impl Request for UpdateOrderRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_request_type_id() -> i32 {
        SpotRequestType::UpdateOrder as i32
    }
}

// cancel order

impl CancelOrderRequest {
    pub fn new(base_asset_id: u64, quote_asset_id: u64, side: u64, order_id: u64) -> Self {
        CancelOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            order_id,
            market_admin: Bytes::from(vec![0]),
        }
    }
}

impl Request for CancelOrderRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_request_type_id() -> i32 {
        SpotRequestType::CancelOrder as i32
    }
}

// EVENTS

// order new

impl SpotOrderNewEvent {
    pub fn new(account: &AccountPubKey, order_id: u64, side: u64, price: u64, quantity: u64) -> Self {
        SpotOrderNewEvent {
            account: Bytes::from(account.as_ref().to_vec()),
            order_id,
            side,
            price,
            quantity,
        }
    }
}

impl Event for SpotOrderNewEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_event_type_id() -> i32 {
        SpotEventType::OrderNew as i32
    }
}

// order fill

impl SpotOrderFillEvent {
    pub fn new(account: &AccountPubKey, order_id: u64, side: u64, price: u64, quantity: u64) -> Self {
        SpotOrderFillEvent {
            account: Bytes::from(account.as_ref().to_vec()),
            order_id,
            side,
            price,
            quantity,
        }
    }
}

impl Event for SpotOrderFillEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_event_type_id() -> i32 {
        SpotEventType::OrderFill as i32
    }
}

// order partial fill

impl SpotOrderPartialFillEvent {
    pub fn new(account: &AccountPubKey, order_id: u64, side: u64, price: u64, quantity: u64) -> Self {
        SpotOrderPartialFillEvent {
            account: Bytes::from(account.as_ref().to_vec()),
            order_id,
            side,
            price,
            quantity,
        }
    }
}

impl Event for SpotOrderPartialFillEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_event_type_id() -> i32 {
        SpotEventType::OrderPartialFill as i32
    }
}

// order update

impl SpotOrderUpdateEvent {
    pub fn new(account: &AccountPubKey, order_id: u64, side: u64, price: u64, quantity: u64) -> Self {
        SpotOrderUpdateEvent {
            account: Bytes::from(account.as_ref().to_vec()),
            order_id,
            side,
            price,
            quantity,
        }
    }
}

impl Event for SpotOrderUpdateEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_event_type_id() -> i32 {
        SpotEventType::OrderUpdate as i32
    }
}

// order cancel

impl SpotOrderCancelEvent {
    pub fn new(account: &AccountPubKey, order_id: u64) -> Self {
        SpotOrderCancelEvent {
            account: Bytes::from(account.as_ref().to_vec()),
            order_id,
        }
    }
}

impl Event for SpotOrderCancelEvent {
    fn get_controller_id() -> i32 {
        ControllerType::Spot as i32
    }
    fn get_event_type_id() -> i32 {
        SpotEventType::OrderCancel as i32
    }
}

// TRANSACTION BUILDERS

pub fn create_create_orderbook_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    base_asset_id: u64,
    quote_asset_id: u64,
) -> Transaction {
    Transaction::new(
        sender,
        recent_block_hash,
        &CreateOrderbookRequest::new(base_asset_id, quote_asset_id),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_market_order_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    quantity: u64,
) -> Transaction {
    Transaction::new(
        sender,
        recent_block_hash,
        &MarketOrderRequest::new(base_asset_id, quote_asset_id, side, quantity),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_limit_order_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
) -> Transaction {
    Transaction::new(
        sender,
        recent_block_hash,
        &LimitOrderRequest::new(base_asset_id, quote_asset_id, side, price, quantity),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_update_order_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    price: u64,
    quantity: u64,
    order_id: u64,
) -> Transaction {
    Transaction::new(
        sender,
        recent_block_hash,
        &UpdateOrderRequest::new(base_asset_id, quote_asset_id, side, price, quantity, order_id),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_cancel_order_transaction(
    sender: &AccountPubKey,
    recent_block_hash: CertificateDigest,
    base_asset_id: u64,
    quote_asset_id: u64,
    side: u64,
    order_id: u64,
) -> Transaction {
    Transaction::new(
        sender,
        recent_block_hash,
        &CancelOrderRequest::new(base_asset_id, quote_asset_id, side, order_id),
    )
}
