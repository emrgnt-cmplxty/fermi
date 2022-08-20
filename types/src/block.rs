use narwhal_executor::SerializedTransaction;
use narwhal_types::CertificateDigest;
use serde::{Deserialize, Serialize};

pub type BlockNumber = u64;
pub type BlockDigest = CertificateDigest;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_digest: BlockDigest,
    pub transactions: Vec<SerializedTransaction>,
}
