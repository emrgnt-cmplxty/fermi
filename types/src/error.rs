use thiserror::Error;

// pub enum GDEXError {
//     AccountCreation(String),
//     AccountLookup(String),
//     BlockValidation(String),
//     PendingBlock(String),
//     OrderBookCreation(String),
//     OrderProc(String),
//     PaymentRequest(String),
//     Vote(String),
//     SignatureVer(String),
// }

#[derive(Debug, Error)]
pub enum GDEXError {
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
