//! 
//! toy consensus model that imitates a simple PoS
//! features still need further fleshing out
//! 
extern crate core;
extern crate proc;
extern crate types;

use super::router::{asset_creation_txn, route_transaction, stake_txn};
use core::{
    block::{Block, BlockContainer, generate_block_hash},
    hash_clock::HashClock,
    transaction::{
        TxnRequest, 
        TxnVariant, 
    },
    vote_cert::VoteCert,
};
use gdex_crypto::{SigningKey, traits::Uniform, hash::{HashValue}};
use proc::{
    bank::BankController,
    stake::StakeController, spot::SpotController,
};
use rand::rngs::ThreadRng;
use types::account::{AccountPubKey, AccountPrivKey, AccountSignature, AccountError};

// specify the number of tokens creator stakes during genesis
const GENESIS_STAKE_AMOUNT: u64 = 1_000_000;

// the consensus manager owns all Controllers and is responsible for
// processing transactions, updating state, and reaching consensus in "toy" conditions
pub struct ConsensusManager
{
    block_container: BlockContainer<TxnVariant>,
    hash_clock: HashClock,
    bank_controller: BankController,
    spot_controller: SpotController,
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
            spot_controller: SpotController::new(),
            stake_controller: StakeController::new(),
            validator_pub_key: account_pub_key,
            validator_private_key: private_key,
        }
    }

    // build the genesis block by creating the base asset and staking some funds
    pub fn build_genesis_block(&mut self) -> Result<Block::<TxnVariant>, AccountError> {
        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

        // GENESIS TXN #0 -> create the base asset of the blockhain
        // <-- there is some hair on this process as we currently issue all tokens at genesis to the initial generator -->
        // --> however, this can be alleviated by extending the functionality of the bank module <--
        let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(self.validator_pub_key, &self.validator_private_key)?;
        route_transaction(self, &signed_txn)?;
        txns.push(signed_txn);

        // GENESIS TXN 1 -> stake some manager funds to allow consensus to begin
        let signed_txn: TxnRequest<TxnVariant> = stake_txn(self.validator_pub_key, &self.validator_private_key, GENESIS_STAKE_AMOUNT)?;
        route_transaction(self, &signed_txn)?;
        txns.push(signed_txn);

        // return the initial genesis block
        self.propose_block(txns)
    }

    // take a list of transactions and create a valid Block w/ the managers vote included
    pub fn propose_block(&self, txns: Vec<TxnRequest<TxnVariant>>) -> Result<Block::<TxnVariant>, AccountError> {
        let block_hash: HashValue = generate_block_hash(&txns);
        let mut vote_cert: VoteCert = VoteCert::new(self.stake_controller.get_staked(&self.validator_pub_key)?, block_hash);

        let vote_response: bool = true;
        self.cast_vote(&mut vote_cert, vote_response)?;

        Ok(Block::<TxnVariant>::new(txns, self.validator_pub_key, block_hash, self.hash_clock.get_time(), vote_cert))
    } 

    // cast a vote on a given block and append a valid signature
    pub fn cast_vote(&self, vote_cert: &mut VoteCert, vote_response: bool) -> Result<(), AccountError> {
        // validator signs the block hash appended to their vote response
        let validator_signature: AccountSignature  = self.validator_private_key.sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(
            self.validator_pub_key, 
            validator_signature, 
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

    pub fn get_spot_controller(&mut self) -> &mut SpotController {
        &mut self.spot_controller
    }

    pub fn get_block_container(&mut self) -> &mut BlockContainer<TxnVariant> {
        &mut self.block_container
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

    // this is necessary because rust does not allow multiple mutable borrows to coexist
    pub fn get_all_controllers(&mut self) -> (&mut BankController,  &mut StakeController, &mut SpotController){
        (&mut self.bank_controller, &mut self.stake_controller, &mut self.spot_controller)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::router::{orderbook_creation_txn, order_transaction, payment_txn};
    use proc::bank::{CREATED_ASSET_BALANCE, PRIMARY_ASSET_ID};
    use types::{asset::AssetId, orderbook::OrderSide};
    
    // specify the number of tokens sent to second validator
    const SECONDARY_SEED_PAYMENT: u64 = 100_000;
    const QUOTE_ASSET_ID: AssetId = 1;

    #[test]
    fn test_simple_consensus() {
        // start w/ a two-validator setup
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let secondary_validator: ConsensusManager = ConsensusManager::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block: Block<TxnVariant> = primary_validator.build_genesis_block().unwrap();

        // validate block immediately as genesis proposer is only staked validator
        genesis_block.validate_block().unwrap();
        primary_validator.get_block_container().append_block(genesis_block);

        // check that the initial funding was successful by checking state of controllers
        let primary_pub_key: AccountPubKey = primary_validator.get_validator_pub_key();
        let primary_balance: u64 = primary_validator.get_bank_controller().get_balance(&primary_pub_key, PRIMARY_ASSET_ID).unwrap();
        assert!(primary_balance == CREATED_ASSET_BALANCE - GENESIS_STAKE_AMOUNT, "Unexpected balance after token genesis & stake");
        let primary_staked: u64 = primary_validator.get_stake_controller().get_staked(&primary_pub_key).unwrap();
        assert!(primary_staked == GENESIS_STAKE_AMOUNT, "Unexpected stake balance after staking primary validator");

        // begin second block where second validator is funded and staked
        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

        // fund second validator
        let signed_txn: TxnRequest<TxnVariant> = payment_txn(
            primary_pub_key,
            primary_validator.get_validator_private_key(),
            secondary_validator.get_validator_pub_key(),
            PRIMARY_ASSET_ID,
            SECONDARY_SEED_PAYMENT
        ).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);

        let secondary_pub_key: AccountPubKey = secondary_validator.get_validator_pub_key();
        let secondary_balance: u64 = primary_validator.get_bank_controller().get_balance(&secondary_pub_key, PRIMARY_ASSET_ID).unwrap();
        assert!(secondary_balance == SECONDARY_SEED_PAYMENT, "Unexpected balance after funding second validator");

        // stake second validator
        let signed_txn: TxnRequest<TxnVariant> = stake_txn(secondary_pub_key, secondary_validator.get_validator_private_key(), SECONDARY_SEED_PAYMENT).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);
        let secondary_staked: u64 = primary_validator.stake_controller.get_staked(&secondary_validator.get_validator_pub_key()).unwrap();
        assert!(secondary_staked == SECONDARY_SEED_PAYMENT, "Unexpected stake after staking secondary validator");

        // process second block
        // tick the internal clock
        // --> this is not yet used in the consensus, but can be used to create a VDF later <--
        primary_validator.tick_hash_clock(1_000);
        let second_block: Block<TxnVariant> = primary_validator.propose_block(txns).unwrap();
        // second validator does not need to vote as staked amount remains significantly less than primary
        second_block.validate_block().unwrap();

        // begin third block - here a second asset will be made and an orderbook inistantiated
        let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();
        let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(secondary_pub_key, secondary_validator.get_validator_private_key()).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);

        let new_asset_balance: u64 = primary_validator.get_bank_controller().get_balance(&secondary_pub_key, QUOTE_ASSET_ID).unwrap();
        assert!(new_asset_balance == CREATED_ASSET_BALANCE, "Unexpected balance after second token genesis");

        // TODO - add order book logic here
        let signed_txn: TxnRequest<TxnVariant> = orderbook_creation_txn(secondary_pub_key, secondary_validator.get_validator_private_key(), PRIMARY_ASSET_ID, QUOTE_ASSET_ID).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);

        let signed_txn: TxnRequest<TxnVariant> = order_transaction(
            primary_pub_key, 
            primary_validator.get_validator_private_key(), 
            PRIMARY_ASSET_ID, 
            QUOTE_ASSET_ID,
            OrderSide::Ask,
            10,
            10,
        ).unwrap();
        route_transaction(&mut primary_validator, &signed_txn).unwrap();
        txns.push(signed_txn);
        // TODO - play around w/ consensus to test it in more scenarios
    }
}