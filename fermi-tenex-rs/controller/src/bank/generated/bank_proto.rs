// REQUESTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateAssetRequest {
    #[prost(uint64, tag="1")]
    pub dummy: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentRequest {
    #[prost(bytes="bytes", tag="1")]
    pub receiver: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub asset_id: u64,
    #[prost(uint64, tag="3")]
    pub quantity: u64,
}
// EVENTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetCreatedEvent {
    #[prost(uint64, tag="1")]
    pub asset_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentSuccessEvent {
    #[prost(bytes="bytes", tag="1")]
    pub sender: ::prost::bytes::Bytes,
    #[prost(bytes="bytes", tag="2")]
    pub receiver: ::prost::bytes::Bytes,
    #[prost(uint64, tag="3")]
    pub asset_id: u64,
    #[prost(uint64, tag="4")]
    pub quantity: u64,
}
// REQUEST ENUM

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BankRequestType {
    CreateAsset = 0,
    Payment = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BankEventType {
    AssetCreated = 0,
    PaymentSuccess = 1,
}
