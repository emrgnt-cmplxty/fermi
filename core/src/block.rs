//!
//! the block contains a list of transactions as well
//! as a associated metadata which relates to consensus
//!
//! TODO
//! 0.) RELOCATE APPROPRIATE TESTS FROM SUITE/CORE TO HERE
extern crate types;
use crate::transaction::{TransactionRequest, TransactionVariant};
use crate::vote_cert::VoteCert;
use gdex_crypto::hash::{CryptoHash, HashValue, ACCUMULATOR_PLACEHOLDER_HASH};
use gdex_crypto::hash::{CryptoHasher, TestOnlyHasher};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use types::{
    account::{AccountPubKey, AccountSignature},
    error::GDEXError,
    hash_clock::HashTime,
};

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

impl<Variant> Default for BlockContainer<Variant>
where
    Variant: Debug + Clone + CryptoHash + Copy,
{
    fn default() -> Self {
        Self::new()
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

    pub fn push_transaction(&mut self, transaction: TransactionRequest<Variant>) {
        self.transactions.push(transaction);
    }

    pub fn append_vote(
        &mut self,
        valdator_pub_key: AccountPubKey,
        validator_signature: AccountSignature,
        vote_response: bool,
        stake: u64,
    ) -> Result<(), GDEXError> {
        self.vote_cert
            .append_vote(valdator_pub_key, validator_signature, vote_response, stake)
    }

    pub fn update_hash_time(&mut self, hash_time: HashTime) {
        self.hash_time = hash_time;
    }
}

pub struct MerkleTreeInternalNode<H> {
    left_child: HashValue,
    right_child: HashValue,
    hasher: PhantomData<H>,
}

impl<H: CryptoHasher> MerkleTreeInternalNode<H> {
    pub fn new(left_child: HashValue, right_child: HashValue) -> Self {
        Self {
            left_child,
            right_child,
            hasher: PhantomData,
        }
    }
}

impl<H: CryptoHasher> CryptoHash for MerkleTreeInternalNode<H> {
    type Hasher = H;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.update(self.left_child.as_ref());
        state.update(self.right_child.as_ref());
        state.finish()
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Position(u64);

impl Position {
    /// pos count start from 0 on each level
    pub fn from_level_and_pos(level: u32, pos: u64) -> Self {
        // precondition!(level < 64);
        // assume!(1u64 << level > 0); // bitwise and integer operations don't mix.
        let level_one_bits = (1u64 << level) - 1;
        let shifted_pos = if level == 63 { 0 } else { pos << (level + 1) };
        Position(shifted_pos | level_one_bits)
    }

    // Opposite of get_left_node_count_from_position.
    pub fn from_leaf_index(leaf_index: u64) -> Self {
        Self::from_level_and_pos(0, leaf_index)
    }

    /// Smearing all the bits starting from MSB with ones
    fn smear_ones_for_u64(v: u64) -> u64 {
        let mut n = v;
        n |= n >> 1;
        n |= n >> 2;
        n |= n >> 4;
        n |= n >> 8;
        n |= n >> 16;
        n |= n >> 32;
        n
    }

    // Given a leaf index, calculate the position of a minimum root which contains this leaf
    /// This method calculates the index of the smallest root which contains this leaf.
    /// Observe that, the root position is composed by a "height" number of ones
    ///
    /// For example
    /// ```text
    ///     0010010(node)
    ///     0011111(smearing)
    ///     -------
    ///     0001111(root)
    /// ```
    pub fn root_from_leaf_index(leaf_index: u64) -> Self {
        let leaf = Self::from_leaf_index(leaf_index);
        Self(Self::smear_ones_for_u64(leaf.0) >> 1)
    }
}

fn compute_parent_hash(left_hash: HashValue, right_hash: HashValue) -> HashValue {
    if left_hash == *ACCUMULATOR_PLACEHOLDER_HASH && right_hash == *ACCUMULATOR_PLACEHOLDER_HASH {
        *ACCUMULATOR_PLACEHOLDER_HASH
    } else {
        MerkleTreeInternalNode::<TestOnlyHasher>::new(left_hash, right_hash).hash()
    }
}

/// Given a list of leaves, constructs the smallest accumulator that has all the leaves and
/// computes the hash of every node in the tree.
fn compute_hashes_for_all_positions(leaves: &[HashValue]) -> HashMap<Position, HashValue> {
    if leaves.is_empty() {
        return HashMap::new();
    }

    let mut current_leaves = leaves.to_vec();
    current_leaves.resize(leaves.len().next_power_of_two(), *ACCUMULATOR_PLACEHOLDER_HASH);
    let mut position_to_hash = HashMap::new();
    let mut current_level = 0;

    while current_leaves.len() > 1 {
        assert!(current_leaves.len().is_power_of_two());

        let mut parent_leaves = vec![];
        for (index, _hash) in current_leaves.iter().enumerate().step_by(2) {
            let left_hash = current_leaves[index];
            let right_hash = current_leaves[index + 1];
            let parent_hash = compute_parent_hash(left_hash, right_hash);
            parent_leaves.push(parent_hash);

            let left_pos = Position::from_level_and_pos(current_level, index as u64);
            let right_pos = Position::from_level_and_pos(current_level, index as u64 + 1);
            assert_eq!(position_to_hash.insert(left_pos, left_hash), None);
            assert_eq!(position_to_hash.insert(right_pos, right_hash), None);
        }

        assert_eq!(current_leaves.len(), parent_leaves.len() << 1);
        current_leaves = parent_leaves;
        current_level += 1;
    }

    assert_eq!(
        position_to_hash.insert(Position::from_level_and_pos(current_level, 0), current_leaves[0],),
        None,
    );
    position_to_hash
}

// Computes the root hash of an accumulator with given leaves.
fn compute_root_hash_naive(leaves: &[HashValue]) -> HashValue {
    let position_to_hash = compute_hashes_for_all_positions(leaves);
    if position_to_hash.is_empty() {
        return *ACCUMULATOR_PLACEHOLDER_HASH;
    }

    let rightmost_leaf_index = leaves.len() as u64 - 1;
    *position_to_hash
        .get(&Position::root_from_leaf_index(rightmost_leaf_index))
        .expect("Root position should exist in the map.")
}

// generate the merkle hash root hash of the block
pub fn generate_block_hash(transactions: &[TransactionRequest<TransactionVariant>]) -> HashValue {
    let transaction_hashes: Vec<HashValue> = transactions.iter().map(|x| x.get_transaction().hash()).collect();
    compute_root_hash_naive(&transaction_hashes[..])
}

#[cfg(test)]
mod tests {
    use super::super::hash_clock::HashClock;
    use super::super::transaction::{
        CreateAssetRequest, TransactionRequest, TransactionVariant, TransactionVariant::CreateAssetTransaction,
    };
    use super::super::vote_cert::VoteCert;
    use super::*;
    use gdex_crypto::{hash::CryptoHash, SigningKey, Uniform};
    use types::{account::AccountPrivKey, spot::DiemCryptoMessage};

    #[test]
    fn test_block_functionality() {
        let private_key = AccountPrivKey::generate_for_testing(0);
        let account_pub_key = (&private_key).into();
        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();

        let transaction = CreateAssetTransaction(CreateAssetRequest {});
        let transaction_hash = transaction.hash();
        let signed_hash = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction =
            TransactionRequest::<TransactionVariant>::new(transaction, account_pub_key, signed_hash);
        signed_transaction.verify_transaction().unwrap();
        transactions.push(signed_transaction);

        let block_hash = generate_block_hash(&transactions);
        let hash_clock = HashClock::default();
        let dummy_vote_cert = VoteCert::new(0, block_hash);

        let block = Block::<TransactionVariant>::new(
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
        assert!(block.get_block_number() == 0, "block number does not match input");

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
        assert!(
            block_container.get_blocks()[0].get_block_number() == block.get_block_number(),
            "block container 0th block number does not match input"
        );

        let mut default_block_container = BlockContainer::default();
        default_block_container.append_block(block.clone());
        assert!(
            default_block_container.get_blocks()[0].get_block_number() == block.get_block_number(),
            "default block container 0th block number does not match input"
        );
    }
}
