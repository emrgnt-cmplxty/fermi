// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
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
    FaucetAirdropRequest,
    FaucetAirdropResponse,
    RelayerBlock,
    RelayerBlockInfoResponse,
    RelayerBlockResponse,
    RelayerGetBlockInfoRequest,
    RelayerGetBlockRequest,
    RelayerGetLatestBlockInfoRequest,
    RelayerGetLatestOrderbookDepthRequest,
    RelayerLatestOrderbookDepthResponse,
    RelayerMetricsRequest,
    RelayerMetricsResponse,
};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/block.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod block;

pub use block::{Block, BlockInfo};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/transaction.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod transaction;

pub use transaction::{ControllerType, RequestType, SignedTransaction, Transaction, Version};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/bank_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod bank_requests;

pub use bank_requests::{CreateAssetRequest, PaymentRequest};

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/spot_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod spot_requests;

pub use spot_requests::{
    CancelOrderRequest, CreateOrderbookRequest, LimitOrderRequest, MarketOrderRequest, UpdateOrderRequest,
};
