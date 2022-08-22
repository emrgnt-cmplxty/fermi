use crate::crypto;
use blake2::VarBlake2b;
use narwhal_crypto::{Digest, Hash};
use narwhal_executor::SerializedTransaction;
use narwhal_types::{Certificate, CertificateDigest};
use serde::{Deserialize, Serialize};
use std::io::Bytes;

pub type BlockNumber = u64;
pub type BlockDigest = CertificateDigest;
pub type BlockCertificate = Certificate;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_certificate: BlockCertificate,
    pub transactions: Vec<SerializedTransaction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockInfo {
    pub block_number: BlockNumber,
    pub block_digest: BlockDigest,
}
