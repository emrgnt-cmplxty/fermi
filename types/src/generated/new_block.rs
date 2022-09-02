// INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewBlock {
    #[prost(bytes="bytes", tag="1")]
    pub block_certificate: ::prost::bytes::Bytes,
    #[prost(message, repeated, tag="2")]
    pub transactions: ::prost::alloc::vec::Vec<super::new_transaction::NewSignedTransaction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewRelayerBlockResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
    #[prost(message, optional, tag="2")]
    pub block: ::core::option::Option<NewBlock>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewBlockInfo {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
    #[prost(bytes="bytes", tag="2")]
    pub block_digest: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewRelayerBlockInfoResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
    #[prost(message, optional, tag="2")]
    pub block_info: ::core::option::Option<NewBlockInfo>,
}
