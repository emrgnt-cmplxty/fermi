//! 
//! TODO
//! 0.) CREATE LOGICAL HANDLING FOR SIGNING & PROCESSING THE SEED TOKEN TXN
//! 1.) BUILD TXN PROCESSING LOGIC WHICH EXECUTES (THIS) CODE BY PROCESSING AN INPUT XN
//! 2.) BUILD TXN CREATION & PROCESSING LOGIC WHICH STORES AND REPLICATES (THIS) LOGIC
//! 3.) ERADICATE EARLY UNWRAPS
//! 
extern crate core;
extern crate proc;
extern crate types;

use std::convert::TryInto;
use rand::rngs::{ThreadRng};

use core::{
    block::{Block, BlockContainer, generate_block_hash},
    hash_clock::{HashClock},
    transaction::{
        TxnRequest, 
        TxnVariant, CreateAsset,
    },
    vote_cert::{VoteCert},
};
use diem_crypto::{
    SigningKey,
    traits::{Uniform},
    hash::{CryptoHash, HashValue},
};
use proc::{
    bank::{BankController},
    stake::{StakeController},
};
use types::{
    account::{AccountPubKey, AccountPrivKey, AccountSignature, AccountError},
    spot::{DiemCryptoMessage},
};

// Specify # of tokens created at genesis
const GENESIS_SEED_PAYMENT: u64 = 1_100_000;
// Specify # of tokens sent to second validator
const VALIDATOR_SEED_PAYMENT: u64 = 100_000;

// TOY CONSENSUS
fn genesis_token_txn(validator_pub_key: AccountPubKey, validator_private_key: &AccountPrivKey) -> Result<TxnRequest<TxnVariant>, AccountError>  {

    let txn: TxnVariant = TxnVariant::CreateAssetTransaction(CreateAsset{});
    let txn_hash: HashValue = txn.hash();
    // TODO #0 //
    let signed_hash: AccountSignature  = (*validator_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    // NOTE, THIS SPECIAL TRANSACATON WOULD FAIL VERIFICATION BECAUSE SIGNING IS INCORRECT 
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            validator_pub_key, 
            signed_hash 
        )
    )
}
pub struct ConsensusManager
{
    block_container: BlockContainer<TxnVariant>,
    hash_clock: HashClock,
    bank_controller: BankController,
    stake_controller: StakeController,
    validator_pub_key: AccountPubKey,
    validator_private_key: AccountPrivKey,
}
impl ConsensusManager {
    pub fn new() -> Self {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        ConsensusManager {
            block_container: BlockContainer::new(),
            hash_clock: HashClock::new(),
            bank_controller: BankController::new(),
            stake_controller: StakeController::new(),
            validator_pub_key: account_pub_key,
            validator_private_key: private_key
        }
    }
    
    pub fn propose_block(&self, txns: Vec<TxnRequest<TxnVariant>>) -> Result<Block::<TxnVariant>, AccountError> {
        let block_hash: HashValue = generate_block_hash(&txns);
        let mut vote_cert: VoteCert = VoteCert::new(self.stake_controller.get_staked(&self.validator_pub_key)?, block_hash);
        let vote_response: bool = true;

        self.cast_vote(&mut vote_cert, vote_response)?;
        Ok(Block::<TxnVariant>::new(txns, self.validator_pub_key, block_hash, self.hash_clock.get_time(), vote_cert))
    } 

    pub fn build_genesis_block(&mut self) -> Result<Block::<TxnVariant>, AccountError> {
        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

        // GENESIS TXN #0
        let signed_txn: TxnRequest<TxnVariant> = genesis_token_txn(self.validator_pub_key, &self.validator_private_key)?;
        txns.push(signed_txn);
        // TODO #1 //
        self.bank_controller.create_asset(&self.validator_pub_key)?;

        // GENESIS TXN 1
        // TODO #3 //
        let validator_staked_amount: u64 = ((GENESIS_SEED_PAYMENT - VALIDATOR_SEED_PAYMENT) as u64).try_into().unwrap();
        self.stake_controller.stake(&mut self.bank_controller, &self.validator_pub_key, validator_staked_amount)?;

        // RETURN FIRST BLOCK
        self.propose_block(txns)
    }

    pub fn cast_vote(&self, vote_cert: &mut VoteCert, vote_response: bool) -> Result<(), AccountError> {
        let validator_signed_hash: AccountSignature  = self.validator_private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(
            self.validator_pub_key, 
            validator_signed_hash, 
            vote_response, 
            self.stake_controller.get_staked(&self.validator_pub_key)?
        )?;
        Ok(())   
    }

    pub fn init_new_consensus(&mut self) -> Result<(), AccountError> {
        let genesis_block: Block<TxnVariant> = self.build_genesis_block()?;
        genesis_block.validate_block();
        self.block_container.append_block(genesis_block);
        Ok(())
    }

    pub fn get_bank_controller(&mut self) -> &mut BankController {
        &mut self.bank_controller
    }

    pub fn get_validator_pub_key(&self) -> AccountPubKey {
        return self.validator_pub_key
    }

    pub fn get_validator_private_key(&self) -> &AccountPrivKey {
        return &self.validator_private_key
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        transaction::{
            CreateAsset,
            TxnVariant::CreateAssetTransaction,
        }
    };
    #[test]
    fn test_consensus() {
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let mut secondary_validator: ConsensusManager = ConsensusManager::new();
        primary_validator.init_new_consensus().unwrap();

        primary_validator.bank_controller.transfer(
            &primary_validator.validator_pub_key, 
            &secondary_validator.validator_pub_key,
            0,
            VALIDATOR_SEED_PAYMENT
        ).unwrap();

        primary_validator.stake_controller.stake(&mut primary_validator.bank_controller, &secondary_validator.validator_pub_key, VALIDATOR_SEED_PAYMENT).unwrap();

        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

        let txn: TxnVariant = CreateAssetTransaction(CreateAsset{});
        let txn_hash: HashValue = txn.hash();
        let signed_hash: AccountSignature  = primary_validator.validator_private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(
            txn,
            primary_validator.validator_pub_key, 
            signed_hash, 
        );
        txns.push(signed_txn);
        // let new_block: Block<TxnVariant> = primary_validator.propose_block(txns).unwrap();
    }
}