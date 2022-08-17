// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg_attr(beta, allow(clippy::derive_partial_eq_without_eq))]
#[path = "generated/gdex.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod gdex;

pub use gdex::{
    transactions_client::TransactionsClient,
    transactions_server::{Transactions, TransactionsServer},
    Empty, Transaction as TransactionProto,
};
