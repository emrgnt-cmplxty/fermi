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
