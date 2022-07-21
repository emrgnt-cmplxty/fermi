//!
//! the block contains a list of transactions as well
//! as a associated metadata which relates to consensus
//!
//! TODO
//! 0.) RELOCATE APPROPRIATE TESTS FROM SUITE/CORE TO HERE
//!
use super::transaction::{TransactionRequest, TransactionVariant};
use super::vote_cert::VoteCert;
use gdex_crypto::hash::{CryptoHash, HashValue};
use std::fmt::Debug;
use types::{account::AccountPubKey, hash_clock::HashTime, spot::DiemCryptoMessage};

#[derive(Clone, Debug)]
pub struct BlockContainer<Variant>
where
    Variant: Debug + Clone + CryptoHash + Copy,
{
    blocks: Vec<Block<Variant>>,
}
impl<Variant> BlockContainer<Variant>
where
    Variant: Debug + Clone + CryptoHash + Copy,
{
    pub fn new() -> Self {
        BlockContainer { blocks: Vec::new() }
    }

    pub fn get_blocks(&self) -> &Vec<Block<Variant>> {
        &self.blocks
    }

    pub fn append_block(&mut self, block: Block<Variant>) {
        self.blocks.push(block);
    }
}

#[derive(Clone, Debug)]
pub struct Block<Variant>
where
    Variant: Debug + Clone + CryptoHash + Copy,
{
    transactions: Vec<TransactionRequest<Variant>>,
    proposer: AccountPubKey,
    block_hash: HashValue,
    hash_time: HashTime,
    vote_cert: VoteCert,
}
impl<Variant> Block<Variant>
where
    Variant: Debug + Clone + CryptoHash + Copy,
{
    pub fn new(
        transactions: Vec<TransactionRequest<Variant>>,
        proposer: AccountPubKey,
        block_hash: HashValue,
        hash_time: HashValue,
        vote_cert: VoteCert,
    ) -> Self {
        Block {
            transactions,
            proposer,
            block_hash,
            hash_time,
            vote_cert,
        }
    }

    pub fn get_transactions(&self) -> &Vec<TransactionRequest<Variant>> {
        &self.transactions
    }

    pub fn get_proposer(&self) -> &AccountPubKey {
        &self.proposer
    }

    pub fn get_block_hash(&self) -> HashValue {
        self.block_hash
    }

    pub fn get_hash_time(&self) -> HashValue {
        self.hash_time
    }

    pub fn get_vote_cert(&self) -> &VoteCert {
        &self.vote_cert
    }
}

impl<Variant> Default for BlockContainer<Variant>
where
    Variant: Debug + Clone + CryptoHash + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

// generate a unique block hash by appending all the hashes transactions inside the block
pub fn generate_block_hash(transactions: &Vec<TransactionRequest<TransactionVariant>>) -> HashValue {
    let mut hash_string = String::from("");
    for transaction in transactions {
        hash_string += &transaction.get_transaction().hash().to_string();
    }
    DiemCryptoMessage(hash_string).hash()
}
