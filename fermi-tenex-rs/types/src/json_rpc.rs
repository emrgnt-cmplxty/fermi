// fermi
use crate::{
    block::{Block, BlockInfo, BlockNumber},
    transaction::QueriedTransaction,
    utils,
};
// mysten
use fastcrypto::Digest;
// external
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default, JsonSchema)]
pub struct BlockReply {
    pub transactions: Vec<QueriedTransaction>,
    pub block_id: String,
}

impl From<Block> for BlockReply {
    fn from(block: Block) -> Self {
        #[allow(clippy::redundant_closure)]
        let transactions = block
            .transactions
            .into_iter()
            .map(|transaction| QueriedTransaction::from(transaction))
            .collect();
        Self {
            transactions,
            block_id: utils::encode_bytes_hex(Digest::from(block.block_certificate.header.id).to_vec()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, JsonSchema)]
pub struct BlockInfoReply {
    pub validator_system_epoch_time_in_micros: u64,
    pub block_number: BlockNumber,
    pub block_id: String,
}

impl From<BlockInfo> for BlockInfoReply {
    fn from(block_info: BlockInfo) -> Self {
        Self {
            block_number: block_info.block_number,
            validator_system_epoch_time_in_micros: block_info.validator_system_epoch_time_in_micros,
            block_id: utils::encode_bytes_hex(Digest::from(block_info.block_digest).to_vec()),
        }
    }
}
