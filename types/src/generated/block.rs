// INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(bytes="bytes", tag="1")]
    pub block_certificate: ::prost::bytes::Bytes,
    #[prost(message, repeated, tag="2")]
    pub transactions: ::prost::alloc::vec::Vec<super::transaction::SignedTransaction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockInfo {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
    #[prost(bytes="bytes", tag="2")]
    pub digest: ::prost::bytes::Bytes,
}
