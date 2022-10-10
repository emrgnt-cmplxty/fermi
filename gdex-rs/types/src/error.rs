// mysten
use narwhal_executor::ExecutionStateError;
// external
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash, JsonSchema)]
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
    #[error("Not implemented")]
    NotImplemented,

    // transaction associated errors
    // TODO - clean up error layout
    #[error("Sender, payload and signature are not consistent")]
    FailedVerification,
    #[error("Futures market initialization failed")]
    FuturesInitialization,
    #[error("Futures market parameters update failed")]
    FuturesUpdate,
    #[error("Insufficient collateral available for withdrawal")]
    FuturesWithdrawal,
    #[error("Marketplace does not exist.")]
    MarketplaceExistence,
    #[error("Orderbook does not exist.")]
    OrderbookExistence,
    #[error("Market existence check failed")]
    MarketExistence,
    #[error("Market updating market prices")]
    MarketPrices,
    #[error("Insufficient collateral for this operation")]
    InsufficientCollateral,
    #[error("Cannot liquidate, target is above minimum collateral threshold")]
    CannotLiquidateTargetCollateral,
    #[error("Cannot liquidate, target still has open orders")]
    CannotLiquidateOpenOrders,
    #[error("Cannot liquidate, target position does not match")]
    CannotLiquidatePosition,
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
    #[error("Failed to process duplicate transaction")]
    TransactionDuplicate,
    // other errors
    #[error("Error while converting type")]
    Conversion,
    // Consensus output errors
    #[error("Failed to execute transaction")]
    ExecError,

    // server errors
    #[error("Failed to process the inbound transaction")]
    RpcFailure(String),

    // proto errors
    #[error("Failed to sign transaction")]
    SigningError,
    #[error("Failed to serialize object")]
    SerializationError,
    #[error("Failed to deserialize object")]
    DeserializationError,

    // controller errors
    #[error("Target controller not found")]
    InvalidControllerError,
    #[error("Controller can not handle request type")]
    InvalidRequestTypeError,

    #[error("Failed to verify transaction signature")]
    TransactionSignatureVerificationError,
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
    FailedVerification(fastcrypto::traits::Error),
    Serialization(Box<bincode::ErrorKind>),
    Deserialization(Box<bincode::ErrorKind>),
}
