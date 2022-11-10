#[path = "generated/services.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod services;
pub use services::{
    // faucet
    faucet_client::FaucetClient,
    faucet_server::{Faucet, FaucetServer},
    validator_grpc_client::ValidatorGrpcClient,
    validator_grpc_server::{ValidatorGrpc, ValidatorGrpcServer},
    BlockInfoRequest,
    BlockInfoResponse,
    BlockRequest,
    BlockResponse,
    Empty,
    FaucetAirdropRequest,
    FaucetAirdropResponse,
    LatestBlockInfoRequest,
    MetricsRequest,
    MetricsResponse,
};

#[path = "generated/transaction.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod transaction;
pub use transaction::*;
