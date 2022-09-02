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
    #[prost(uint64, tag="5")]
    pub local_timestamp: u64,
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
    #[prost(uint64, tag="6")]
    pub local_timestamp: u64,
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
    pub local_timestamp: u64,
    #[prost(uint64, tag="7")]
    pub order_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOrderRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
    #[prost(uint64, tag="3")]
    pub quantity: u64,
    #[prost(uint64, tag="4")]
    pub local_timestamp: u64,
    #[prost(uint64, tag="5")]
    pub order_id: u64,
}
