// crate
use crate::error::GDEXError;
use crate::transaction::ExecutionResultBody;

// mysten
use narwhal_executor::SerializedTransaction;
use narwhal_types::{Certificate, CertificateDigest};

// external
use serde::{Deserialize, Serialize};

pub type BlockNumber = u64;
pub type BlockDigest = CertificateDigest;
pub type BlockCertificate = Certificate;

// TODO we define this in many places, centralize
type ExecutionResult = Result<ExecutionResultBody, GDEXError>;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Block {
    pub block_certificate: BlockCertificate,
    pub transactions: Vec<(SerializedTransaction, ExecutionResult)>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct BlockInfo {
    pub block_number: BlockNumber,
    pub block_digest: BlockDigest,
    // TODO - change to consensus time when implemented
    pub validator_system_epoch_time_in_micros: u64,
}
