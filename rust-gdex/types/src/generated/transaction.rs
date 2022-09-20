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
    #[prost(enumeration="ControllerType", tag="3")]
    pub target_controller: i32,
    #[prost(enumeration="RequestType", tag="4")]
    pub request_type: i32,
    #[prost(bytes="bytes", tag="5")]
    pub recent_block_hash: ::prost::bytes::Bytes,
    #[prost(uint64, tag="6")]
    pub fee: u64,
    #[prost(bytes="bytes", tag="7")]
    pub request_bytes: ::prost::bytes::Bytes,
}
// ENUMS

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ControllerType {
    Bank = 0,
    Stake = 1,
    Spot = 2,
    Consensus = 3,
    Futures = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RequestType {
    /// begin bank messages
    Payment = 0,
    /// begin spot market messages
    CreateAsset = 1,
    CreateOrderbook = 2,
    MarketOrder = 3,
    LimitOrder = 4,
    UpdateOrder = 5,
    CancelOrder = 6,
    /// begin futures market messages
    CreateMarketplace = 7,
    CreateMarket = 8,
    UpdateMarketParams = 9,
    UpdateTime = 10,
    UpdatePrices = 11,
    AccountDeposit = 12,
    AccountWithdrawal = 13,
    FuturesMarketOrder = 14,
    FuturesLimitOrder = 15,
    FuturesUpdateOrder = 16,
    FuturesCancelOrder = 17,
}
