use std::{fmt::Debug};

use super::transaction::{TxnRequest, TxnVariant};
use super::vote_cert::{VoteCert};
use diem_crypto::{
    hash::{CryptoHash, HashValue},
};
use types::{
    account::{AccountPubKey, AccountError},
    spot::{DiemCryptoMessage},
};

pub struct BlockContainer <Variant>
    where Variant : Debug + Clone + CryptoHash + Copy
{
    blocks: Vec<Block<Variant>>,
}
impl <Variant> BlockContainer <Variant> 
    where Variant : Debug + Clone + CryptoHash + Copy
{
    pub fn new() -> Self {
        BlockContainer {
            blocks: Vec::new(),
        }
    }

    pub fn get_blocks(&self) -> &Vec<Block<Variant>> {
        &self.blocks
    }

    pub fn append_block(&mut self, block: Block<Variant>) {
        self.blocks.push(block);
    }
}

pub struct Block <Variant>
    where Variant : Debug + Clone + CryptoHash + Copy
{
    txns: Vec<TxnRequest<Variant>>,
    proposer: AccountPubKey,
    block_hash: HashValue,
    clock_hash: HashValue,
    vote_cert: VoteCert
}
impl <Variant> Block <Variant> 
    where Variant : Debug + Clone + CryptoHash + Copy
{
    pub fn new(
        txns: Vec<TxnRequest<Variant>>, 
        proposer: AccountPubKey, 
        block_hash: HashValue, 
        clock_hash: HashValue,
        vote_cert: VoteCert
    ) -> Self {
        Block {
            txns,
            proposer,
            block_hash,
            clock_hash,
            vote_cert
        }
    }

    pub fn get_txns(&self) -> &Vec<TxnRequest<Variant>> {
        &self.txns
    }

    pub fn get_proposer(&self) -> &AccountPubKey {
        &self.proposer
    }

    pub fn get_block_hash(&self) -> HashValue {
        self.block_hash
    }

    pub fn get_clock_hash(&self) -> HashValue {
        self.clock_hash
    }

    pub fn get_vote_cert(&self) -> &VoteCert {
        &self.vote_cert
    }

    pub fn validate_block(&self) -> Result<(), AccountError> {
        if self.get_vote_cert().vote_has_passed() {
            Ok(())
        } else {
            Err(AccountError::BlockValidation("Validation failed".to_string()))
        }
    }
}

pub fn generate_block_hash(txns: &Vec<TxnRequest<TxnVariant>>) -> HashValue {
    let mut hash_string = String::from("");
    for txn in txns {
        hash_string += &txn.get_txn().hash().to_string();
    }
    DiemCryptoMessage(hash_string).hash()
}
