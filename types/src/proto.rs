// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/gdex.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod gdex;

pub use gdex::{
    faucet_client::FaucetClient,
    faucet_server::{Faucet, FaucetServer},
    relayer_client::RelayerClient,
    relayer_server::{Relayer, RelayerServer},
    transactions_client::TransactionsClient,
    transactions_server::{Transactions, TransactionsServer},
    Block as BlockProto,
    BlockInfo as BlockInfoProto,
    Empty,
    FaucetAirdropRequest,
    FaucetAirdropResponse,
    RelayerBlockInfoResponse,
    RelayerBlockResponse,
    RelayerGetBlockInfoRequest,
    RelayerGetBlockRequest,
    RelayerGetLatestBlockInfoRequest,
    Transaction as TransactionProto,
};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/new_block.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod new_block;

pub use new_block:: {
    NewBlock,
    NewRelayerBlockResponse,
    NewBlockInfo,
    NewRelayerBlockInfoResponse,
};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/new_transaction.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod new_transaction;

pub use new_transaction::{
    Controller,
    RequestType,
    Version,
    NewSignedTransaction,
    NewTransaction,
};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/bank_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod bank_requests;

pub use bank_requests::{
    CreateAssetRequest,
    PaymentRequest,
};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/spot_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod spot_requests;

pub use spot_requests::{  
    CreateOrderbookRequest,
    MarketOrderRequest,
    LimitOrderRequest,
    UpdateOrderRequest,
    CancelOrderRequest,
};
