// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::error::GDEXError;

#[path = "generated/services.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod services;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    FuturesOrder,
    FuturesPosition,
    RelayerBlock,
    RelayerBlockInfoResponse,
    RelayerBlockResponse,
    RelayerFuturesMarketsResponse,
    RelayerFuturesUserResponse,
    RelayerGetBlockInfoRequest,
    RelayerGetBlockRequest,
    RelayerGetFuturesMarketsRequest,
    RelayerGetFuturesUserRequest,
    RelayerGetLatestBlockInfoRequest,
    RelayerGetLatestCatchupStateRequest,
    RelayerGetLatestOrderbookDepthRequest,
    RelayerLatestCatchupStateResponse,
    RelayerLatestOrderbookDepthResponse,
    RelayerMetricsRequest,
    RelayerMetricsResponse,
};

impl Serialize for FuturesPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = super::transaction::serialize_protobuf(self);
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for FuturesPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let position: Result<FuturesPosition, GDEXError> = super::transaction::deserialize_protobuf(&bytes);
        match position {
            Ok(p) => Ok(p),
            Err(e) => Err(serde::de::Error::custom(e.to_string())),
        }
    }
}

impl Serialize for FuturesOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = super::transaction::serialize_protobuf(self);
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for FuturesOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let order: Result<FuturesOrder, GDEXError> = super::transaction::deserialize_protobuf(&bytes);
        match order {
            Ok(o) => Ok(o),
            Err(e) => Err(serde::de::Error::custom(e.to_string())),
        }
    }
}
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
