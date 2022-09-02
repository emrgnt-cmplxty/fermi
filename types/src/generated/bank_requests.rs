// REQUESTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateAssetRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentRequest {
    #[prost(bytes="bytes", tag="1")]
    pub receiver: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub asset_id: u64,
    #[prost(uint64, tag="3")]
    pub amount: u64,
}
