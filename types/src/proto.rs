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
    BlockInfo as BlockInfoProto, Empty, FaucetAirdropRequest, FaucetAirdropResponse, RelayerRequest, RelayerResponse,
    Transaction, Transaction as TransactionProto,
};
