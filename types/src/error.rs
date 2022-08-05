//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcError {
    #[error("Account already exists")]
    AccountCreation,
    #[error("Failed to find account")]
    AccountLookup,
    #[error("Payment request failed")]
    PaymentRequest,
    #[error("Order request failed")]
    OrderRequest,
    #[error("Orderbook creation failed")]
    OrderBookCreation,
}

#[derive(Debug)]
pub enum SignedTransactionError {
    FailedVerification(narwhal_crypto::traits::Error),
    Serialization(Box<bincode::ErrorKind>),
    Deserialization(Box<bincode::ErrorKind>),
}

#[macro_export]
macro_rules! fp_bail {
    ($e:expr) => {
        return Err($e)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fp_ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            fp_bail!($e);
        }
    };
}

pub type SuiError = sui_types::error::SuiError;
pub type SuiResult<T = ()> = Result<T, SuiError>;
