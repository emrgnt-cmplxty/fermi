//!
//! the block contains a list of transactions as well
//! as a associated metadata which relates to consensus
//!
//! TODO
//! 0.) RELOCATE APPROPRIATE TESTS FROM SUITE/CORE TO HERE
extern crate types;
use crate::transaction::{TransactionRequest, TransactionVariant};
use crate::vote_cert::VoteCert;
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
    block_number: u64,
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
        block_number: u64,
        hash_time: HashValue,
        vote_cert: VoteCert,
    ) -> Self {
        Block {
            transactions,
            proposer,
            block_hash,
            block_number,
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

    pub fn get_block_number(&self) -> u64 {
        self.block_number
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

#[cfg(test)]
mod tests {
    use super::super::hash_clock::HashClock;
    use super::super::transaction::{
        CreateAssetRequest, TransactionRequest, TransactionVariant, TransactionVariant::CreateAssetTransaction,
    };
    use super::super::vote_cert::VoteCert;
    use super::*;
    use gdex_crypto::{
        hash::{CryptoHash, HashValue},
        SigningKey, Uniform,
    };
    use types::account::{AccountPrivKey, AccountSignature};

    #[test]
    fn test_block_functionality() {
        let private_key: AccountPrivKey = AccountPrivKey::generate_for_testing();
        let account_pub_key: AccountPubKey = (&private_key).into();
        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();

        let transaction: TransactionVariant = CreateAssetTransaction(CreateAssetRequest {});
        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<TransactionVariant> =
            TransactionRequest::<TransactionVariant>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
        transactions.push(signed_transaction);

        let block_hash: HashValue = generate_block_hash(&transactions);
        let hash_clock: HashClock = HashClock::default();
        let dummy_vote_cert: VoteCert = VoteCert::new(0, block_hash);

        let block: Block<TransactionVariant> = Block::<TransactionVariant>::new(
            transactions.clone(),
            account_pub_key,
            block_hash,
            0,
            hash_clock.get_hash_time(),
            dummy_vote_cert.clone(),
        );
        assert!(
            block.get_proposer() == &account_pub_key,
            "block validator does not match input"
        );
        assert!(block.get_block_hash() == block_hash, "block hash does not match input");
        assert!(
            block.get_transactions().len() == transactions.len(),
            "block transaction length does not match input"
        );
        assert!(
            block.get_hash_time() == hash_clock.get_hash_time(),
            "block hash time does not match input"
        );
        assert!(
            block.get_vote_cert().get_block_hash() == dummy_vote_cert.get_block_hash(),
            "block vote cert block hash does not match input"
        );

        let mut block_container: BlockContainer<TransactionVariant> = BlockContainer::new();
        block_container.append_block(block.clone());
        assert!(
            block_container.get_blocks().len() == 1,
            "Incorrect length of blocks in container"
        );

        assert!(
            block_container.get_blocks()[0].get_proposer() == block.get_proposer(),
            "block container 0th block validator does not match input"
        );
        assert!(
            block_container.get_blocks()[0].get_block_hash() == block.get_block_hash(),
            "block container 0th block hash does not match input"
        );
        assert!(
            block_container.get_blocks()[0].get_transactions().len() == block.get_transactions().len(),
            "block transaction length does not match input"
        );
        assert!(
            block_container.get_blocks()[0].get_hash_time() == block.get_hash_time(),
            "block container 0th block hash time does not match input"
        );
        assert!(
            block_container.get_blocks()[0].get_vote_cert().get_block_hash() == block.get_vote_cert().get_block_hash(),
            "block container 0th block vote cert block hash does not match input"
        );
    }
}
