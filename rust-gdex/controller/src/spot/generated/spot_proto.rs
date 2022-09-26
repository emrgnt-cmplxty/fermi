// REQUESTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateOrderbookRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketOrderRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LimitOrderRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
    #[prost(uint64, tag="5")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateOrderRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
    #[prost(uint64, tag="5")]
    pub quantity: u64,
    #[prost(uint64, tag="6")]
    pub order_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOrderRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub order_id: u64,
    #[prost(bytes="bytes", tag="5")]
    pub market_admin: ::prost::bytes::Bytes,
}
// EVENTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotOrderNewEvent {
    #[prost(bytes="bytes", tag="1")]
    pub account: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub order_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
    #[prost(uint64, tag="5")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotOrderFillEvent {
    #[prost(bytes="bytes", tag="1")]
    pub account: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub order_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
    #[prost(uint64, tag="5")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotOrderPartialFillEvent {
    #[prost(bytes="bytes", tag="1")]
    pub account: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub order_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
    #[prost(uint64, tag="5")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotOrderUpdateEvent {
    #[prost(bytes="bytes", tag="1")]
    pub account: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub order_id: u64,
    #[prost(uint64, tag="3")]
    pub side: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
    #[prost(uint64, tag="5")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpotOrderCancelEvent {
    #[prost(bytes="bytes", tag="1")]
    pub account: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub order_id: u64,
}
// ENUMS

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpotRequestType {
    CreateOrderbook = 0,
    MarketOrder = 1,
    LimitOrder = 2,
    UpdateOrder = 3,
    CancelOrder = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpotEventType {
    OrderNew = 0,
    OrderFill = 1,
    OrderPartialFill = 2,
    OrderUpdate = 3,
    OrderCancel = 4,
}
