//! Copyright (c) 2021, Facebook, Inc. and its affiliates
//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! Note, the code in this file is taken almost directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/committee.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
use crate::{
    account::{ValidatorPubKey, ValidatorPubKeyBytes},
    error::{GDEXError, GDEXResult},
    fp_ensure,
};
use itertools::Itertools;
use rand_latest::rngs::OsRng;
use rand_latest::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub type ValidatorName = ValidatorPubKeyBytes;

pub type EpochId = u64;

pub type StakeUnit = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Committee {
    pub epoch: EpochId,
    pub voting_rights: Vec<(ValidatorName, StakeUnit)>,
    pub total_votes: StakeUnit,
    // Note: this is a derived structure, no need to store.
    #[serde(skip)]
    expanded_keys: HashMap<ValidatorName, ValidatorPubKey>,
    #[serde(skip)]
    index_map: HashMap<ValidatorName, usize>,
    #[serde(skip)]
    loaded: bool,
}

impl Committee {
    pub fn new(epoch: EpochId, voting_rights: BTreeMap<ValidatorName, StakeUnit>) -> GDEXResult<Self> {
        let mut voting_rights: Vec<(ValidatorName, StakeUnit)> = voting_rights.iter().map(|(a, s)| (*a, *s)).collect();

        fp_ensure!(
            !voting_rights.is_empty(),
            GDEXError::InvalidCommittee("committee has 0 members".into())
        );

        fp_ensure!(
            voting_rights.iter().any(|(_, s)| *s != 0),
            GDEXError::InvalidCommittee("at least one committee member must have non-zero stake.".into())
        );

        voting_rights.sort_by_key(|(a, _)| *a);
        let total_votes = voting_rights.iter().map(|(_, votes)| *votes).sum();

        let (expanded_keys, index_map) = Self::load_inner(&voting_rights);

        Ok(Committee {
            epoch,
            voting_rights,
            total_votes,
            expanded_keys,
            index_map,
            loaded: true,
        })
    }

    // We call this if these have not yet been computed
    pub fn load_inner(
        voting_rights: &[(ValidatorName, StakeUnit)],
    ) -> (HashMap<ValidatorName, ValidatorPubKey>, HashMap<ValidatorName, usize>) {
        let expanded_keys: HashMap<ValidatorName, ValidatorPubKey> = voting_rights
            .iter()
            // TODO: Verify all code path to make sure we always have valid public keys.
            // e.g. when a new validator is registering themself on-chain.
            .map(|(addr, _)| (*addr, (*addr).try_into().expect("Invalid Validator Key")))
            .collect();

        let index_map: HashMap<ValidatorName, usize> = voting_rights
            .iter()
            .enumerate()
            .map(|(index, (addr, _))| (*addr, index))
            .collect();
        (expanded_keys, index_map)
    }

    pub fn reload_fields(&mut self) {
        let (expanded_keys, index_map) = Committee::load_inner(&self.voting_rights);
        self.expanded_keys = expanded_keys;
        self.index_map = index_map;
        self.loaded = true;
    }

    pub fn validator_index(&self, author: &ValidatorName) -> Option<u32> {
        if !self.loaded {
            return self
                .voting_rights
                .iter()
                .position(|(a, _)| a == author)
                .map(|i| i as u32);
        }
        self.index_map.get(author).map(|i| *i as u32)
    }

    pub fn validator_by_index(&self, index: u32) -> Option<&ValidatorName> {
        self.voting_rights.get(index as usize).map(|(name, _)| name)
    }

    pub fn epoch(&self) -> EpochId {
        self.epoch
    }

    pub fn public_key(&self, validator: &ValidatorName) -> GDEXResult<ValidatorPubKey> {
        match self.expanded_keys.get(validator) {
            // TODO: Check if this is unnecessary copying.
            Some(v) => Ok(v.clone()),
            None => (*validator)
                .try_into()
                .map_err(|_| GDEXError::InvalidCommittee(format!("Validator #{} not found", validator))),
        }
    }

    /// Samples authorities by weight
    pub fn sample(&self) -> &ValidatorName {
        // unwrap safe unless committee is empty
        Self::choose_multiple_weighted(&self.voting_rights[..], 1)
            .next()
            .unwrap()
    }

    fn choose_multiple_weighted(
        slice: &[(ValidatorName, StakeUnit)],
        count: usize,
    ) -> impl Iterator<Item = &ValidatorName> {
        // unwrap is safe because we validate the committee composition in `new` above.
        // See https://docs.rs/rand/latest/rand/distributions/weighted/enum.WeightedError.html
        // for possible errors.
        slice
            .choose_multiple_weighted(&mut OsRng, count, |(_, weight)| *weight as f64)
            .unwrap()
            .map(|(a, _)| a)
    }

    pub fn shuffle_by_stake(
        &self,
        // try these authorities first
        preferences: Option<&BTreeSet<ValidatorName>>,
        // only attempt from these authorities.
        restrict_to: Option<&BTreeSet<ValidatorName>>,
    ) -> Vec<ValidatorName> {
        let restricted = self
            .voting_rights
            .iter()
            .filter(|(name, _)| {
                if let Some(restrict_to) = restrict_to {
                    restrict_to.contains(name)
                } else {
                    true
                }
            })
            .cloned();

        let (preferred, rest): (Vec<_>, Vec<_>) = if let Some(preferences) = preferences {
            restricted.partition(|(name, _)| preferences.contains(name))
        } else {
            (Vec::new(), restricted.collect())
        };

        Self::choose_multiple_weighted(&preferred, preferred.len())
            .chain(Self::choose_multiple_weighted(&rest, rest.len()))
            .cloned()
            .collect()
    }

    pub fn weight(&self, author: &ValidatorName) -> StakeUnit {
        match self.voting_rights.binary_search_by_key(author, |(a, _)| *a) {
            Err(_) => 0,
            Ok(idx) => self.voting_rights[idx].1,
        }
    }

    pub fn quorum_threshold(&self) -> StakeUnit {
        // If N = 3f + 1 + k (0 <= k < 3)
        // then (2 N + 3) / 3 = 2f + 1 + (2k + 2)/3 = 2f + 1 + k = N - f
        2 * self.total_votes / 3 + 1
    }

    pub fn validity_threshold(&self) -> StakeUnit {
        // If N = 3f + 1 + k (0 <= k < 3)
        // then (N + 2) / 3 = f + 1 + k/3 = f + 1
        (self.total_votes + 2) / 3
    }

    /// Given a sequence of (ValidatorName, value) for values, provide the
    /// value at the particular threshold by stake. This orders all provided values
    /// in ascending order and pick the appropriate value that has under it threshold
    /// stake. You may use the function `validity_threshold` or `quorum_threshold` to
    /// pick the f+1 (1/3 stake) or 2f+1 (2/3 stake) thresholds respectively.
    ///
    /// This function may be used in a number of settings:
    /// - When we pass in a set of values produced by authorities with at least 2/3 stake
    ///   and pick a validity_threshold it ensures that the resulting value is either itself
    ///   or is in between values provided by an honest node.
    /// - When we pass in values associated with the totality of stake and set a threshold
    ///   of quorum_threshold, we ensure that at least a majority of honest nodes (ie >1/3
    ///   out of the 2/3 threshold) have a value smaller than the value returned.
    pub fn robust_value<A, V>(&self, items: impl Iterator<Item = (A, V)>, threshold: StakeUnit) -> (ValidatorName, V)
    where
        A: Borrow<ValidatorName> + Ord,
        V: Ord,
    {
        debug_assert!(threshold < self.total_votes);

        let items = items.map(|(a, v)| (v, self.weight(a.borrow()), *a.borrow())).sorted();
        let mut total = 0;
        for (v, s, a) in items {
            total += s;
            if threshold <= total {
                return (a, v);
            }
        }
        unreachable!();
    }

    pub fn num_members(&self) -> usize {
        self.voting_rights.len()
    }

    pub fn members(&self) -> impl Iterator<Item = &(ValidatorName, StakeUnit)> {
        self.voting_rights.iter()
    }

    pub fn names(&self) -> impl Iterator<Item = &ValidatorName> {
        self.voting_rights.iter().map(|(name, _)| name)
    }

    pub fn stakes(&self) -> impl Iterator<Item = StakeUnit> + '_ {
        self.voting_rights.iter().map(|(_, stake)| *stake)
    }

    pub fn validator_exists(&self, name: &ValidatorName) -> bool {
        self.voting_rights.binary_search_by_key(name, |(a, _)| *a).is_ok()
    }
}

impl PartialEq for Committee {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch && self.voting_rights == other.voting_rights && self.total_votes == other.total_votes
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::account::ValidatorKeyPair;
    use crate::crypto::{get_key_pair, KeypairTraits};

    #[test]
    #[should_panic]
    fn empty_committee_fail() {
        let authorities = BTreeMap::new();
        let _committee = Committee::new(0, authorities).unwrap();
    }

    #[test]
    #[should_panic]
    fn unstaked_committee() {
        let mut authorities = BTreeMap::new();
        let (_, sec1): (_, ValidatorKeyPair) = get_key_pair();
        let a1: ValidatorName = sec1.public().into();
        authorities.insert(a1, 0);

        let _committee = Committee::new(0, authorities).unwrap();
    }

    #[test]
    fn single_committee_workflow() {
        let mut authorities = BTreeMap::new();
        let (_, sec1): (_, ValidatorKeyPair) = get_key_pair();
        let a1: ValidatorName = sec1.public().into();
        authorities.insert(a1, 1);

        let mut committee = Committee::new(0, authorities.clone()).unwrap();
        // check we can reload fields
        committee.reload_fields();

        // check validator index functionality
        let index = committee.validator_index(&a1);
        assert!(index.unwrap() == 0);

        let validator = committee.sample();
        // sample should only return the single input validator
        assert!(*validator == ValidatorPubKeyBytes::from(a1.clone()));

        assert!(committee.num_members() == 1);

        let mut members = committee.members();
        let member = members.next();
        // assert that members first returns the initial validator
        assert!(member.unwrap().0.clone() == ValidatorPubKeyBytes::from(a1.clone()));
        let member = members.next();
        // assert that members returns None
        assert!(member == None);

        let mut names = committee.names();
        let name = names.next();
        // assert that names first returns the initial validator
        assert!(name.unwrap().clone() == ValidatorPubKeyBytes::from(a1.clone()));
        let name = names.next();
        // assert that names next returns None
        assert!(name == None);

        let mut stakes = committee.stakes();
        let stake = stakes.next();
        // assert that stakes first returns the initial validator staked (1)
        assert!(stake.unwrap().clone() == 1);
        let stake = names.next();
        // assert that stakes next returns None
        assert!(stake == None);

        // check validator exists workflow
        assert!(committee.validator_exists(&a1));
        let (_, sec2): (_, ValidatorKeyPair) = get_key_pair();
        let a2: ValidatorName = sec2.public().into();

        assert!(!committee.validator_exists(&a2));

        // check equality workflow
        let committee_copy = Committee::new(0, authorities).unwrap();
        assert!(committee == committee_copy);
    }

    #[test]
    fn test_shuffle_by_weight() {
        let (_, sec1): (_, ValidatorKeyPair) = get_key_pair();
        let (_, sec2): (_, ValidatorKeyPair) = get_key_pair();
        let (_, sec3): (_, ValidatorKeyPair) = get_key_pair();
        let a1: ValidatorName = sec1.public().into();
        let a2: ValidatorName = sec2.public().into();
        let a3: ValidatorName = sec3.public().into();

        let mut authorities = BTreeMap::new();
        authorities.insert(a1, 1);
        authorities.insert(a2, 1);
        authorities.insert(a3, 1);

        let committee = Committee::new(0, authorities).unwrap();

        assert_eq!(committee.shuffle_by_stake(None, None).len(), 3);

        let mut pref = BTreeSet::new();
        pref.insert(a2);

        // preference always comes first
        for _ in 0..100 {
            assert_eq!(a2, *committee.shuffle_by_stake(Some(&pref), None).first().unwrap());
        }

        let mut restrict = BTreeSet::new();
        restrict.insert(a2);

        for _ in 0..100 {
            let res = committee.shuffle_by_stake(None, Some(&restrict));
            assert_eq!(1, res.len());
            assert_eq!(a2, res[0]);
        }

        // empty preferences are valid
        let res = committee.shuffle_by_stake(Some(&BTreeSet::new()), None);
        assert_eq!(3, res.len());

        let res = committee.shuffle_by_stake(None, Some(&BTreeSet::new()));
        assert_eq!(0, res.len());
    }

    #[test]
    fn test_robust_value() {
        let (_, sec1): (_, ValidatorKeyPair) = get_key_pair();
        let (_, sec2): (_, ValidatorKeyPair) = get_key_pair();
        let (_, sec3): (_, ValidatorKeyPair) = get_key_pair();
        let (_, sec4): (_, ValidatorKeyPair) = get_key_pair();
        let a1: ValidatorName = sec1.public().into();
        let a2: ValidatorName = sec2.public().into();
        let a3: ValidatorName = sec3.public().into();
        let a4: ValidatorName = sec4.public().into();

        let mut authorities = BTreeMap::new();
        authorities.insert(a1, 1);
        authorities.insert(a2, 1);
        authorities.insert(a3, 1);
        authorities.insert(a4, 1);
        let committee = Committee::new(0, authorities).unwrap();
        let items = vec![(a1, 666), (a2, 1), (a3, 2), (a4, 0)];
        assert_eq!(
            committee.robust_value(items.into_iter(), committee.quorum_threshold()),
            (a3, 2)
        );

        let items = vec![(a1, "a"), (a2, "b"), (a3, "c"), (a4, "d")];
        assert_eq!(
            committee.robust_value(items.into_iter(), committee.quorum_threshold()),
            (a3, "c")
        );
    }
}
