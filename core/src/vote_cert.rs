//! TODO
//! 0.) BUILD CUSTOM ERROR TYPES
//! 1.) GET RID OF LAZY PANIC CHECKS
//! 2.) INCLUDE BLOCK NUMBER
//! 3.) INCLUDE VOTE HASH & SIGNATURE
//! 

use std::{
    collections::HashMap,
};


use types::{
    account::{AccountPubKey},
};

pub const DEFAULT_QUORUM_THRESHOLD: f64 = 0.05;

    // TODO #2 & # 3 //
    pub struct Vote
{
    pub response: bool,
    pub stake: u64
}


pub struct VoteCert
{
    pub votes: HashMap<AccountPubKey, Vote>,
    pub quorum_threshold: f64,
    pub total_voted: u64,
    pub total_staked: u64
}

impl VoteCert {
    pub fn new(total_staked: u64) -> Self {
        VoteCert {
            votes: HashMap::new(),
            quorum_threshold: DEFAULT_QUORUM_THRESHOLD,
            total_voted: 0,
            total_staked
        }
    }

    // TODO #0 //
    pub fn append_vote(&mut self, account_pub_key: &AccountPubKey, response: bool, stake: u64) -> Result<(), String> {
        if self.votes.contains_key(&account_pub_key) {
            Err("Vote already exists!".to_string())
        } else {
            self.votes.insert(*account_pub_key, Vote { response, stake });
            self.total_voted += stake;
            Ok(())
        }
    }
    
    pub fn reached_quorum(&self) -> bool {
        (self.total_voted as f64 / self.total_staked as f64) > self.quorum_threshold
    }
}

#[cfg(test)]
mod tests {
    use diem_crypto::{
        Uniform,
    };
    use types::{
        account::{AccountPrivKey},
    };
    
    use super::*;
    #[test]
    fn valid_vote() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        const TOTAL_STAKED: u64 = 5_000;

        let mut vote_cert = VoteCert::new(TOTAL_STAKED);

        const FIRST_STAKED: u64 = 100;
        vote_cert.append_vote(&account_pub_key, true, FIRST_STAKED).unwrap();
        assert!(!vote_cert.reached_quorum());

        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        const SECOND_STAKED: u64 = 1_000;
        vote_cert.append_vote(&account_pub_key, false, SECOND_STAKED).unwrap();
        assert!(vote_cert.reached_quorum());
    }

    // TODO #1 //
    #[test]
    #[should_panic]
    fn failed_vote() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        const TOTAL_STAKED: u64 = 5_000;

        let mut vote_cert = VoteCert::new(TOTAL_STAKED);

        const FIRST_STAKED: u64 = 100;
        vote_cert.append_vote(&account_pub_key, true, FIRST_STAKED).unwrap();
        vote_cert.append_vote(&account_pub_key, true, FIRST_STAKED).unwrap();
    }

}