// REQUESTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMarketplaceRequest {
    #[prost(uint64, tag="1")]
    pub quote_asset_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMarketRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateMarketParamsRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub max_leverage: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateTimeRequest {
    #[prost(uint64, tag="1")]
    pub latest_time: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdatePricesRequest {
    #[prost(uint64, repeated, tag="1")]
    pub latest_prices: ::prost::alloc::vec::Vec<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountDepositRequest {
    #[prost(int64, tag="1")]
    pub quantity: i64,
    #[prost(bytes="bytes", tag="2")]
    pub market_admin: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountWithdrawalRequest {
    #[prost(uint64, tag="1")]
    pub quantity: u64,
    #[prost(bytes="bytes", tag="2")]
    pub market_admin: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuturesLimitOrderRequest {
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
    #[prost(bytes="bytes", tag="6")]
    pub market_admin: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelAllRequest {
    #[prost(bytes="bytes", tag="1")]
    pub target: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidateRequest {
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
    #[prost(bytes="bytes", tag="6")]
    pub market_admin: ::prost::bytes::Bytes,
    #[prost(bytes="bytes", tag="7")]
    pub target: ::prost::bytes::Bytes,
}
