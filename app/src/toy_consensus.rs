//! 
//! TODO
//! 0.) CREATE LOGICAL HANDLING FOR SIGNING & PROCESSING THE SEED TOKEN TXN
//! 1.) ERADICATE EARLY UNWRAPS
//! 
extern crate core;
extern crate proc;
extern crate types;

use rand::rngs::{ThreadRng};
use super::{
    router::{route_transaction}
};
use core::{
    block::{Block, BlockContainer, generate_block_hash},
    hash_clock::{HashClock},
    transaction::{
        CreateAsset,
        Payment,
        Stake,
        TxnRequest, 
        TxnVariant, 
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

// Specify # of tokens creator stakes at genesis
const GENESIS_STAKE_AMOUNT: u64 = 1_000_000;

// TOY CONSENSUS
fn asset_creation_txn(sender_pub_key: AccountPubKey, sender_private_key: &AccountPrivKey) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::CreateAssetTransaction(CreateAsset{});
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*sender_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
            signed_hash 
        )
    )
}

fn stake_txn(validator_pub_key: AccountPubKey, validator_private_key: &AccountPrivKey, amount: u64) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::StakeAssetTransaction(Stake::new(validator_pub_key, amount));
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*validator_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            validator_pub_key, 
            signed_hash 
        )
    )
}

fn payment_txn(
    sender_pub_key: AccountPubKey, 
    sender_private_key: &AccountPrivKey, 
    receiver_pub_key: AccountPubKey, 
    asset_id: u64,
    amount: u64
) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::PaymentTransaction(Payment::new(sender_pub_key, receiver_pub_key, asset_id, amount));
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*sender_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
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
            validator_private_key: private_key,
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
        // CREATE BASE ASSET OF BLOCKCHAIN
        // ~~ Clearly there is some hair on this process as we are assuming all tokens at genesis go to our primary validator ~~
        // ~~ note that currently no further tokens can be issued ~~
        // ~~ this can be alleviated quite easily by extending the bank module ~~
        let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(self.validator_pub_key, &self.validator_private_key)?;
        route_transaction(self, &signed_txn)?;
        // push successful transaction
        txns.push(signed_txn);

        // GENESIS TXN 1
        // STAKE FUNDS FOR VALIDATION
        let signed_txn: TxnRequest<TxnVariant> = stake_txn(self.validator_pub_key, &self.validator_private_key, GENESIS_STAKE_AMOUNT)?;
        route_transaction(self, &signed_txn)?;
        txns.push(signed_txn);

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

    pub fn get_bank_controller(&mut self) -> &mut BankController {
        &mut self.bank_controller
    }

    pub fn get_stake_controller(&mut self) -> &mut StakeController {
        &mut self.stake_controller
    }

    pub fn get_block_container(&mut self) -> &mut BlockContainer<TxnVariant> {
        &mut self.block_container
    }

    // necessary for instances where we need controllers that are dependent on one another
    // as rust does not allow multiple mutable borrows to coexist
    pub fn get_all_controllers(&mut self) -> (&mut BankController,  &mut StakeController){
        (&mut self.bank_controller, &mut self.stake_controller)
    }

    pub fn get_validator_pub_key(&self) -> AccountPubKey {
        self.validator_pub_key
    }

    pub fn get_validator_private_key(&self) -> &AccountPrivKey {
        &self.validator_private_key
    }

    pub fn tick_hash_clock(&mut self, n_ticks: u64) {
        self.hash_clock.tick(n_ticks);
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    // Specify # of tokens sent to second validator
    const SECONDARY_SEED_PAYMENT: u64 = 100_000;
    use proc::{
        bank::{CREATED_ASSET_BALANCE, STAKE_ASSET_ID},
    };

    #[test]
    fn test_consensus() {
        // two-validator setup
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let secondary_validator: ConsensusManager = ConsensusManager::new();

        // initiate new consensus by creating genesis block 
        // this process funds the primary validator
        let genesis_block: Block<TxnVariant> = primary_validator.build_genesis_block().unwrap();
        // we can validate immediately as genesis proposer is only staked validator
        genesis_block.validate_block().unwrap();
        primary_validator.get_block_container().append_block(genesis_block);

        // check funding was successful
        let primary_pub_key: AccountPubKey = primary_validator.get_validator_pub_key();
        let primary_balance: u64 = primary_validator.get_bank_controller().get_balance(&primary_pub_key, STAKE_ASSET_ID).unwrap();
        assert!(primary_balance == CREATED_ASSET_BALANCE - GENESIS_STAKE_AMOUNT, "Unexpected balance after token genesis & stake");
        let primary_staked: u64 = primary_validator.get_stake_controller().get_staked(&primary_pub_key).unwrap();
        assert!(primary_staked == GENESIS_STAKE_AMOUNT, "Unexpected stake balance after staking primary validator");

        // begin second block by funding and staking second validator
        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

        // fund second validator
        let signed_txn: TxnRequest<TxnVariant> = payment_txn(
            primary_pub_key,
            primary_validator.get_validator_private_key(),
            secondary_validator.get_validator_pub_key(),
            STAKE_ASSET_ID,
            SECONDARY_SEED_PAYMENT
        ).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);

        let secondary_pub_key: AccountPubKey = secondary_validator.get_validator_pub_key();
        let secondary_balance: u64 = primary_validator.get_bank_controller().get_balance(&secondary_pub_key, STAKE_ASSET_ID).unwrap();
        assert!(secondary_balance == SECONDARY_SEED_PAYMENT, "Unexpected balance after funding second validator");

        // stake from second validator
        let signed_txn: TxnRequest<TxnVariant> = stake_txn(secondary_pub_key, &secondary_validator.get_validator_private_key(), SECONDARY_SEED_PAYMENT).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);
        let secondary_staked: u64 = primary_validator.stake_controller.get_staked(&secondary_validator.get_validator_pub_key()).unwrap();

        assert!(secondary_staked == SECONDARY_SEED_PAYMENT, "Unexpected stake after staking secondary validator");

        // PROCESS SECOND BLOCK
        // FIRST TICK INTERNAL CLOCK
        // ~~ this is not yet used in the codebase, but will be consumed later ~~
        primary_validator.tick_hash_clock(1_000);
        let second_block: Block<TxnVariant> = primary_validator.propose_block(txns).unwrap();
        // second validator does not need to vote as his stake is still small
        second_block.validate_block().unwrap();


        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();
        let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(secondary_pub_key, &secondary_validator.get_validator_private_key()).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);

        let new_asset_balance: u64 = primary_validator.get_bank_controller().get_balance(&secondary_pub_key, STAKE_ASSET_ID+1).unwrap();
        assert!(new_asset_balance == CREATED_ASSET_BALANCE, "Unexpected balance after second token genesis");

        // TODO - add order book logic here
        // TODO - play around w/ consensus to create failures

    }
}