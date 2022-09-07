#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(bytes="bytes", tag="1")]
    pub transaction: ::prost::bytes::Bytes,
}
