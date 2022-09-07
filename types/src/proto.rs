// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/services.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod services;

pub use services::{
    Empty,
    // transaction submitter
    transaction_submitter_client::TransactionSubmitterClient,
    transaction_submitter_server::{TransactionSubmitter, TransactionSubmitterServer},
    // faucet
    faucet_client::FaucetClient,
    faucet_server::{Faucet, FaucetServer},
    FaucetAirdropRequest,
    FaucetAirdropResponse,
    // relayer
    relayer_client::RelayerClient,
    relayer_server::{Relayer, RelayerServer},
    RelayerBlock,
    RelayerBlockInfo,
    RelayerBlockInfoResponse,
    RelayerBlockResponse,
    RelayerGetBlockInfoRequest,
    RelayerGetBlockRequest,
    RelayerGetLatestBlockInfoRequest,
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
    ControllerType,
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
