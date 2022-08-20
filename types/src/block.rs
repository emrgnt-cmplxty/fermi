use narwhal_types::CertificateDigest;
use serde::{Deserialize, Serialize};
use narwhal_executor:: SerializedTransaction;

pub type BlockNumber = u64;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub certificate_digest: CertificateDigest,
    pub transactions: Vec<SerializedTransaction>,
}
