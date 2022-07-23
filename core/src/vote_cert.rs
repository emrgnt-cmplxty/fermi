//!
//! vote certificates are used to certify that a block
//! has passed through consensus and garnered sufficient votes
//!
//! TODO
//! 0.) BUILD CUSTOM ERROR TYPES
//! 1.) GET RID OF LAZY PANIC CHECKS
//! 2.) INCLUDE BLOCK NUMBER
//! 3.) INCLUDE VOTE HASH & SIGNATURE
//! 4.) Move compute_vote_output to dedicated type
//!
use gdex_crypto::{hash::HashValue, Signature};
use std::collections::HashMap;
use types::{
    account::{AccountPubKey, AccountSignature},
    error::GDEXError,
    spot::DiemCryptoMessage,
};

// default fraction of staked amount that must vote for a quorum to be reached
pub const DEFAULT_QUORUM_THRESHOLD: f64 = 0.66;
// default fraction of positive votes necessary for block to pass
pub const DEFAULT_VOTE_THRESHOLD: f64 = 0.50;

// TODO #2 & # 3 //
#[derive(Clone, Debug)]

pub struct Vote {
    vote_response: bool,
    stake: u64,
    // signature should correspond to concat of VoteCert block_hash + vote_response
    validator_signature: AccountSignature,
}
impl Vote {
    pub fn get_vote_response(&self) -> bool {
        self.vote_response
    }

    pub fn get_stake(&self) -> u64 {
        self.stake
    }

    pub fn get_validator_signature(&self) -> &AccountSignature {
        &self.validator_signature
    }
}

#[derive(Clone, Debug)]
pub struct VoteCert {
    // map of validator addresses to vote result
    votes: HashMap<AccountPubKey, Vote>,
    quorum_threshold: f64,
    vote_threshold: f64,
    total_voted: u64,
    total_votes_for: u64,
    total_staked: u64,
    block_hash: HashValue,
}

impl VoteCert {
    pub fn new(total_staked: u64, block_hash: HashValue) -> Self {
        VoteCert {
            votes: HashMap::new(),
            quorum_threshold: DEFAULT_QUORUM_THRESHOLD,
            vote_threshold: DEFAULT_VOTE_THRESHOLD,
            total_voted: 0,
            total_votes_for: 0,
            total_staked,
            block_hash,
        }
    }

    // TODO #0 //
    pub fn append_vote(
        &mut self,
        valdator_pub_key: AccountPubKey,
        validator_signature: AccountSignature,
        vote_response: bool,
        stake: u64,
    ) -> Result<(), GDEXError> {
        let vote_msg: &DiemCryptoMessage = &self.compute_vote_msg(vote_response);
        // verify validator signed this block with submitted response
        match validator_signature.verify(vote_msg, &valdator_pub_key) {
            Ok(()) => {
                if let std::collections::hash_map::Entry::Vacant(e) = self.votes.entry(valdator_pub_key) {
                    e.insert(Vote {
                        vote_response,
                        stake,
                        validator_signature,
                    });
                    self.total_voted += stake;
                    self.total_votes_for += if vote_response { stake } else { 0 };
                    Ok(())
                } else {
                    Err(GDEXError::Vote("Vote already exists!".to_string()))
                }
            }
            Err(_) => Err(GDEXError::Vote("Failed to verify signature".to_string())),
        }
    }

    pub fn get_votes(&self) -> &HashMap<AccountPubKey, Vote> {
        &self.votes
    }

    pub fn reached_quorum(&self) -> bool {
        (self.total_voted as f64 / self.total_staked as f64) > self.quorum_threshold
    }

    pub fn vote_has_passed(&self) -> bool {
        (self.total_votes_for as f64 / self.total_voted as f64) > self.vote_threshold && self.reached_quorum()
    }

    pub fn update_quorum_threshold(&mut self, new_threshold: f64) {
        self.quorum_threshold = new_threshold;
    }

    pub fn update_vote_threshold(&mut self, new_threshold: f64) {
        self.vote_threshold = new_threshold;
    }

    pub fn compute_vote_msg(&self, vote_response: bool) -> DiemCryptoMessage {
        DiemCryptoMessage(self.block_hash.to_string() + &vote_response.to_string())
    }

    pub fn get_block_hash(&self) -> HashValue {
        self.block_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use gdex_crypto::{hash::CryptoHash, SigningKey, Uniform};
    use types::account::AccountPrivKey;

    const TOTAL_STAKED: u64 = 1_100;
    const FIRST_STAKED: u64 = 100;
    const SECOND_STAKED: u64 = 1_000;

    #[test]
    fn valid_vote() {
        let private_key = AccountPrivKey::generate_for_testing(0);
        let account_pub_key = (&private_key).into();

        let block_hash = DiemCryptoMessage("".to_string()).hash();
        let mut vote_cert = VoteCert::new(TOTAL_STAKED, block_hash);

        let vote_response = true;
        let signed_hash = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert
            .append_vote(account_pub_key, signed_hash.clone(), true, FIRST_STAKED)
            .unwrap();
        assert!(!vote_cert.reached_quorum());

        let votes: &HashMap<AccountPubKey, Vote> = vote_cert.get_votes();

        // check that validators vote exists
        let vote: Vote = votes
            .get(&account_pub_key)
            .ok_or_else(|| GDEXError::AccountLookup("Failed to find account".to_string()))
            .unwrap()
            .clone();

        assert!(
            vote.get_vote_response() == vote_response,
            "vote response in vote cert does not match input"
        );
        assert!(
            vote.get_stake() == FIRST_STAKED,
            "staked in vote cert does not match input"
        );
        assert!(
            vote.get_validator_signature().clone() == signed_hash,
            "signature in vote cert does not match input"
        );

        let private_key = AccountPrivKey::generate_for_testing(1);
        let account_pub_key = (&private_key).into();
        println!("new account pub key = {}", account_pub_key);
        let vote_response = true;
        let signed_hash = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert
            .append_vote(account_pub_key, signed_hash, vote_response, SECOND_STAKED)
            .unwrap();
        assert!(vote_cert.reached_quorum(), "quorum should have been achieved");
        assert!(vote_cert.vote_has_passed(), "vote should have passed");

        vote_cert.update_vote_threshold(2.);
        assert!(!vote_cert.vote_has_passed(), "vote threshold should fail after update");
        vote_cert.update_quorum_threshold(2.);
        assert!(!vote_cert.reached_quorum(), "quorum fail should fail after update");
    }

    // TODO #1 //
    #[test]
    #[should_panic]
    fn double_vote_failure() {
        let private_key = AccountPrivKey::generate_for_testing(0);
        let account_pub_key = (&private_key).into();

        let block_hash = DiemCryptoMessage("".to_string()).hash();
        let mut vote_cert = VoteCert::new(TOTAL_STAKED, block_hash);

        let vote_response = true;
        let signed_hash = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert
            .append_vote(account_pub_key, signed_hash, true, FIRST_STAKED)
            .unwrap();
        // attempt to re-append same vote
        let signed_hash = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert
            .append_vote(account_pub_key, signed_hash, true, FIRST_STAKED)
            .unwrap();
    }
    // TODO #1 //
    #[test]
    #[should_panic]
    fn invalid_signature_failure() {
        let private_key = AccountPrivKey::generate_for_testing(0);
        let private_key_2 = AccountPrivKey::generate_for_testing(1);
        let account_pub_key = (&private_key).into();

        let block_hash = DiemCryptoMessage("".to_string()).hash();
        let mut vote_cert = VoteCert::new(TOTAL_STAKED, block_hash);

        let vote_response: bool = true;
        // sign with incorrect signature
        let signed_hash = private_key_2.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert
            .append_vote(account_pub_key, signed_hash, true, FIRST_STAKED)
            .unwrap();
    }
}
