use crate::error::GDEXError;
use narwhal_executor::SerializedTransaction;
use narwhal_types::{Certificate, CertificateDigest};
use serde::{Deserialize, Serialize};

pub type BlockNumber = u64;
pub type BlockDigest = CertificateDigest;
pub type BlockCertificate = Certificate;

type ExecutionResult = Result<(), GDEXError>;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Block {
    pub block_certificate: BlockCertificate,
    pub transactions: Vec<(SerializedTransaction, ExecutionResult)>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct BlockInfo {
    pub block_number: BlockNumber,
    pub block_digest: BlockDigest,
    pub validator_system_epoch_time_in_micros: u64,
}
