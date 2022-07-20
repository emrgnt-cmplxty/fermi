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
use gdex_crypto::{Signature, hash::HashValue};
use std::collections::HashMap;
use types::{
    account::{AccountPubKey, AccountSignature, AccountError},
    spot::{DiemCryptoMessage},
};

// default fraction of staked amount that must vote for a quorum to be reached
pub const DEFAULT_QUORUM_THRESHOLD: f64 = 0.05;
// default fraction of positive votes necessary for block to pass
pub const DEFAULT_VOTE_THRESHOLD: f64 = 0.50;

    // TODO #2 & # 3 //
    pub struct Vote
{
    vote_response: bool,
    stake: u64,
    // signature should correspond to concat of VoteCert block_hash + vote_response
    validator_signature: AccountSignature,
}
impl Vote {
    pub fn get_vote_response(&mut self) -> bool {
        self.vote_response
    }

    pub fn get_stake(&mut self) -> u64 {
        self.stake
    }

    pub fn get_validator_signature(&mut self) -> &AccountSignature {
        &self.validator_signature
    }
}

pub struct VoteCert
{
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
        stake: u64
    ) -> Result<(), AccountError> {
        let vote_msg: &DiemCryptoMessage = &self.compute_vote_msg(vote_response);
        // verify validator signed this block with submitted response
        match validator_signature.verify(vote_msg, &valdator_pub_key) {
            Ok(()) => { 
                if self.votes.contains_key(&valdator_pub_key) {
                    Err(AccountError::Vote("Vote already exists!".to_string()))
                } else {
                    self.votes.insert(valdator_pub_key, Vote {vote_response, stake, validator_signature});
                    self.total_voted += stake;
                    self.total_votes_for += if vote_response == true { stake } else { 0 };
                    Ok(())
                }
            },
            Err(_) => { return Err(AccountError::Vote("Failed to verify signature".to_string())); }
        }
    }
    
    pub fn reached_quorum(&self) -> bool {
        (self.total_voted as f64 / self.total_staked as f64) > self.quorum_threshold
    }

    pub fn vote_has_passed(&self) -> bool {
        self.total_votes_for as f64 / self.total_voted as f64 > self.vote_threshold && self.reached_quorum()
    }

    pub fn set_quorum_threshold(&mut self, new_threshold: f64) {
        self.quorum_threshold = new_threshold;
    }

    pub fn set_vote_threshold(&mut self, new_threshold: f64) {
        self.vote_threshold = new_threshold;
    }

    pub fn compute_vote_msg(&self, vote_response: bool) -> DiemCryptoMessage {
        DiemCryptoMessage(self.block_hash.to_string() + &vote_response.to_string())
    }
}

#[cfg(test)]
mod tests {
    use gdex_crypto::{
        SigningKey,
        Uniform,
        hash::{CryptoHash},
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
        let block_hash = DiemCryptoMessage("".to_string()).hash();
        let mut vote_cert = VoteCert::new(TOTAL_STAKED, block_hash);

        const FIRST_STAKED: u64 = 100;
        let vote_response: bool = true;
        let signed_hash: AccountSignature  = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(account_pub_key, signed_hash, true, FIRST_STAKED).unwrap();
        assert!(!vote_cert.reached_quorum());

        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        const SECOND_STAKED: u64 = 1_000;
        let vote_response: bool = true;
        let signed_hash: AccountSignature  = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(account_pub_key, signed_hash, vote_response, SECOND_STAKED).unwrap();
        assert!(vote_cert.reached_quorum());
        assert!(vote_cert.vote_has_passed());

    }

    // TODO #1 //
    #[test]
    #[should_panic]
    fn failed_vote() {
        let mut rng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        const TOTAL_STAKED: u64 = 5_000;

        let block_hash = DiemCryptoMessage("".to_string()).hash();
        let mut vote_cert = VoteCert::new(TOTAL_STAKED, block_hash);

        const FIRST_STAKED: u64 = 100;
        let vote_response: bool = true;
        let signed_hash: AccountSignature  = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(account_pub_key, signed_hash, true, FIRST_STAKED).unwrap();
        let signed_hash: AccountSignature  = private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(account_pub_key, signed_hash, true, FIRST_STAKED).unwrap();
    }

}