// INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutedTransaction {
    #[prost(bytes="bytes", tag="1")]
    pub digest: ::prost::bytes::Bytes,
    #[prost(string, tag="2")]
    pub result: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub signed_transaction: ::core::option::Option<super::transaction::SignedTransaction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, repeated, tag="1")]
    pub executed_transactions: ::prost::alloc::vec::Vec<ExecutedTransaction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockInfo {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
    #[prost(bytes="bytes", tag="2")]
    pub digest: ::prost::bytes::Bytes,
}
