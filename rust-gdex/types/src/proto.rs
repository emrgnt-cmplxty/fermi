// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[path = "generated/services.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod services;

pub use services::{
    // faucet
    faucet_client::FaucetClient,
    faucet_server::{Faucet, FaucetServer},
    // relayer
    relayer_client::RelayerClient,
    relayer_server::{Relayer, RelayerServer},
    // transaction submitter
    transaction_submitter_client::TransactionSubmitterClient,
    transaction_submitter_server::{TransactionSubmitter, TransactionSubmitterServer},
    Depth,
    Empty,
    FuturesPosition,
    FaucetAirdropRequest,
    FaucetAirdropResponse,
    RelayerBlock,
    RelayerBlockInfoResponse,
    RelayerBlockResponse,
    RelayerFuturesPositionsResponse,
    RelayerGetBlockInfoRequest,
    RelayerGetBlockRequest,
    RelayerGetFuturesPositionsRequest,
    RelayerGetLatestBlockInfoRequest,
    RelayerGetLatestOrderbookDepthRequest,
    RelayerLatestOrderbookDepthResponse,
    RelayerMetricsRequest,
    RelayerMetricsResponse,
};

#[path = "generated/block.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod block;

pub use block::*;

#[path = "generated/transaction.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod transaction;

pub use transaction::*;
