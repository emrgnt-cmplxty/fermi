//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use async_trait::async_trait;
use narwhal_executor::ExecutionStateError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash)]
pub enum GDEXError {
    // committee associated errors
    #[error("Invalid committee composition")]
    InvalidCommittee(String),
    #[error("Invalid address")]
    InvalidAddress,

    // controller associated errors
    #[error("Account already exists")]
    AccountCreation,
    #[error("Failed to find account")]
    AccountLookup,
    #[error("Failed to find asset")]
    AssetLookup,

    // transaction associated errors
    #[error("Sender, payload and signature are not consistent")]
    FailedVerification,
    #[error("Order request failed")]
    OrderRequest,
    #[error("Orderbook creation failed")]
    OrderBookCreation,
    #[error("Insufficient balance to place order")]
    OrderExceedsBalance,
    #[error("Payment request failed")]
    PaymentRequest,
    #[error("Failed to serialize the signed transaction")]
    TransactionSerialization,
    #[error("Failed to deserialize into a signed transaction")]
    TransactionDeserialization,

    // Consensus output errors
    #[error("Failed to execute transaction")]
    ExecError,

    // server errors
    #[error("Failed to process the inbound transaction")]
    RpcFailure(String),
}

impl From<tonic::Status> for GDEXError {
    fn from(status: tonic::Status) -> Self {
        Self::RpcFailure(status.message().to_owned())
    }
}

#[async_trait]
impl ExecutionStateError for GDEXError {
    // TODO - implement node error
    fn node_error(&self) -> bool {
        false
    }
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/error.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[macro_export]
macro_rules! fp_bail {
    ($e:expr) => {
        return Err($e)
    };
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/error.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[macro_export(local_inner_macros)]
macro_rules! fp_ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            fp_bail!($e);
        }
    };
}

#[macro_export]
macro_rules! exit_main {
    ($result:expr) => {
        match $result {
            Ok(..) => (),
            Err(err) => {
                println!("{}", err.to_string().bold().red());
                std::process::exit(1);
            }
        }
    };
}

pub type GDEXResult<T = ()> = Result<T, GDEXError>;

#[derive(Debug)]
pub enum SignedTransactionError {
    FailedVerification(narwhal_crypto::traits::Error),
    Serialization(Box<bincode::ErrorKind>),
    Deserialization(Box<bincode::ErrorKind>),
}
