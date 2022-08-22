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
    relay_client::RelayClient,
    relay_server::{Relay, RelayServer},
    transactions_client::TransactionsClient,
    transactions_server::{Transactions, TransactionsServer},
    BlockInfo as BlockInfoProto, Empty, FaucetAirdropRequest, FaucetAirdropResponse, RelayRequest, RelayResponse,
    Transaction, Transaction as TransactionProto,
};
