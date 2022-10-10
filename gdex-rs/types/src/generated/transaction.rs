// STRUCTS

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Version {
    #[prost(uint32, tag="1")]
    pub major: u32,
    #[prost(uint32, tag="2")]
    pub minor: u32,
    #[prost(uint32, tag="3")]
    pub patch: u32,
}
// SIGNED TRANSACTION INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedTransaction {
    #[prost(message, optional, tag="1")]
    pub transaction: ::core::option::Option<Transaction>,
    #[prost(bytes="bytes", tag="2")]
    pub signature: ::prost::bytes::Bytes,
}
// TRANSACTION INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag="1")]
    pub version: ::core::option::Option<Version>,
    #[prost(bytes="bytes", tag="2")]
    pub sender: ::prost::bytes::Bytes,
    #[prost(int32, tag="3")]
    pub target_controller: i32,
    #[prost(int32, tag="4")]
    pub request_type: i32,
    #[prost(bytes="bytes", tag="5")]
    pub recent_block_hash: ::prost::bytes::Bytes,
    #[prost(uint64, tag="6")]
    pub fee: u64,
    #[prost(bytes="bytes", tag="7")]
    pub request_bytes: ::prost::bytes::Bytes,
}
