//!
//! A minimal set of code to implement merkle root hashes
//! originated from Diem MerkleAccumulator, but greatly trimmed
//! to nothing but the bare essentials for this calculation
//!
use gdex_crypto::hash::{CryptoHash, CryptoHasher, HashValue, TestOnlyHasher, ACCUMULATOR_PLACEHOLDER_HASH};
use std::marker::PhantomData;
use std::{collections::HashMap, fmt::Debug};

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
pub fn compute_root_hash_naive(leaves: &[HashValue]) -> HashValue {
    let position_to_hash = compute_hashes_for_all_positions(leaves);
    if position_to_hash.is_empty() {
        return *ACCUMULATOR_PLACEHOLDER_HASH;
    }

    let rightmost_leaf_index = leaves.len() as u64 - 1;
    *position_to_hash
        .get(&Position::root_from_leaf_index(rightmost_leaf_index))
        .expect("Root position should exist in the map.")
}
