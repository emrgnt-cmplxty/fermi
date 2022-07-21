#[derive(Debug)]
pub enum GDEXError {
    AccountCreation(String),
    AccountLookup(String),
    BlockValidation(String),
    OrderBookCreation(String),
    OrderProc(String),
    PaymentRequest(String),
    Vote(String),
    SignatureVer(String),
}
