//!
//! toy consensus model that imitates a simple PoS
//! features still need further fleshing out
//!
extern crate core;
extern crate proc;
extern crate types;

use super::router::{asset_creation_transaction, route_transaction, stake_transaction};
use core::{
    block::{generate_block_hash, Block, BlockContainer},
    hash_clock::{HashClock, TICKS_PER_CYCLE},
    transaction::{TransactionRequest, TransactionVariant},
    vote_cert::VoteCert,
};
use gdex_crypto::{hash::HashValue, traits::Uniform, SigningKey};
use proc::{bank::BankController, spot::SpotController, stake::StakeController};
use rand::rngs::ThreadRng;
use types::{
    account::{AccountError, AccountPrivKey, AccountPubKey, AccountSignature},
    hash_clock::HashTime,
};

// specify the number of tokens creator stakes during genesis
const GENESIS_STAKE_AMOUNT: u64 = 1_000_000;

// the consensus manager owns all Controllers and is responsible for
// processing transactions, updating state, and reaching consensus in "toy" conditions
pub struct ConsensusManager {
    block_container: BlockContainer<TransactionVariant>,
    bank_controller: BankController,
    spot_controller: SpotController,
    stake_controller: StakeController,
    ticks_per_cycle: u64,
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
            bank_controller: BankController::new(),
            spot_controller: SpotController::new(),
            stake_controller: StakeController::new(),
            ticks_per_cycle: TICKS_PER_CYCLE,
            validator_pub_key: account_pub_key,
            validator_private_key: private_key,
        }
    }

    // build the genesis block by creating the base asset and staking some funds
    pub fn build_genesis_block(&mut self) -> Result<Block<TransactionVariant>, AccountError> {
        // create and tick the initial hash clock
        let mut hash_clock: HashClock = HashClock::default();
        hash_clock.cycle();

        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();

        // GENESIS transaction #0 -> create the base asset of the blockhain
        // <-- there is some hair on this process as we currently issue all tokens at genesis to the initial generator -->
        // --> however, this can be alleviated by extending the functionality of the bank module <--
        let signed_transaction: TransactionRequest<TransactionVariant> =
            asset_creation_transaction(self.validator_pub_key, &self.validator_private_key)?;
        route_transaction(self, &signed_transaction)?;
        transactions.push(signed_transaction);

        // GENESIS transaction 1 -> stake some manager funds to allow consensus to begin
        let signed_transaction: TransactionRequest<TransactionVariant> = stake_transaction(
            self.validator_pub_key,
            &self.validator_private_key,
            GENESIS_STAKE_AMOUNT,
        )?;
        route_transaction(self, &signed_transaction)?;
        transactions.push(signed_transaction);

        // return the initial genesis block
        self.propose_block(transactions, hash_clock.get_hash_time())
    }

    // take a list of transactions and create a valid Block w/ the managers vote included
    pub fn propose_block(
        &self,
        transactions: Vec<TransactionRequest<TransactionVariant>>,
        time: HashValue,
    ) -> Result<Block<TransactionVariant>, AccountError> {
        let block_hash: HashValue = generate_block_hash(&transactions);
        let mut vote_cert: VoteCert =
            VoteCert::new(self.stake_controller.get_staked(&self.validator_pub_key)?, block_hash);

        let vote_response: bool = true;
        self.cast_vote(&mut vote_cert, vote_response)?;

        Ok(Block::<TransactionVariant>::new(
            transactions,
            self.validator_pub_key,
            block_hash,
            self.block_container.get_blocks().len() as u64 + 1,
            time,
            vote_cert,
        ))
    }

    // cast a vote on a given block and append a valid signature
    pub fn cast_vote(&self, vote_cert: &mut VoteCert, vote_response: bool) -> Result<(), AccountError> {
        // validator signs the block hash appended to their vote response
        let validator_signature: AccountSignature = self
            .validator_private_key
            .sign(&vote_cert.compute_vote_msg(vote_response));
        vote_cert.append_vote(
            self.validator_pub_key,
            validator_signature,
            vote_response,
            self.stake_controller.get_staked(&self.validator_pub_key)?,
        )?;
        Ok(())
    }

    pub fn validate_and_store_block(
        &mut self,
        block: Block<TransactionVariant>,
        prev_hash_time: HashTime,
    ) -> Result<(), AccountError> {
        let mut hash_clock: HashClock = HashClock::new(prev_hash_time, self.ticks_per_cycle);
        hash_clock.cycle();
        if block.get_vote_cert().vote_has_passed() && block.get_hash_time() == hash_clock.get_hash_time() {
            self.block_container.append_block(block);
            Ok(())
        } else {
            Err(AccountError::BlockValidation("Validation failed".to_string()))
        }
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

    pub fn get_block_container(&self) -> &BlockContainer<TransactionVariant> {
        &self.block_container
    }

    pub fn get_validator_pub_key(&self) -> AccountPubKey {
        self.validator_pub_key
    }

    pub fn get_validator_private_key(&self) -> &AccountPrivKey {
        &self.validator_private_key
    }

    pub fn get_ticks_per_cycle(&self) -> u64 {
        self.ticks_per_cycle
    }

    // this is necessary because rust does not allow multiple mutable borrows to coexist
    pub fn get_all_controllers(&mut self) -> (&mut BankController, &mut StakeController, &mut SpotController) {
        (
            &mut self.bank_controller,
            &mut self.stake_controller,
            &mut self.spot_controller,
        )
    }
}

impl Default for ConsensusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::router::{order_transaction, orderbook_creation_transaction, payment_transaction};
    use super::*;
    use core::hash_clock::HASH_TIME_INIT_MSG;
    use gdex_crypto::hash::CryptoHash;
    use proc::bank::{CREATED_ASSET_BALANCE, PRIMARY_ASSET_ID};
    use types::{asset::AssetId, orderbook::OrderSide, spot::DiemCryptoMessage};
    // specify the number of tokens sent to second validator
    const SECONDARY_SEED_PAYMENT: u64 = 100_000;
    const QUOTE_ASSET_ID: AssetId = 1;

    #[test]
    fn test_simple_consensus() {
        // start w/ a two-validator setup
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let secondary_validator: ConsensusManager = ConsensusManager::new();

        // tick the internal clock
        // --> this is not yet used in the consensus, but can be used to create a VDF later <--

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block: Block<TransactionVariant> = primary_validator.build_genesis_block().unwrap();
        let genesis_hash_time: HashValue = genesis_block.get_hash_time();

        // validate block immediately as genesis proposer is only staked validator
        primary_validator
            .validate_and_store_block(genesis_block, DiemCryptoMessage(HASH_TIME_INIT_MSG.to_string()).hash())
            .unwrap();

        // check that the initial funding was successful by checking state of controllers
        let primary_pub_key: AccountPubKey = primary_validator.get_validator_pub_key();
        let primary_balance: u64 = primary_validator
            .get_bank_controller()
            .get_balance(&primary_pub_key, PRIMARY_ASSET_ID)
            .unwrap();
        assert!(
            primary_balance == CREATED_ASSET_BALANCE - GENESIS_STAKE_AMOUNT,
            "Unexpected balance after token genesis & stake"
        );
        let primary_staked: u64 = primary_validator
            .get_stake_controller()
            .get_staked(&primary_pub_key)
            .unwrap();
        assert!(
            primary_staked == GENESIS_STAKE_AMOUNT,
            "Unexpected stake balance after staking primary validator"
        );

        // begin second block where second validator is funded and staked
        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();
        // initialize clock with time at last block
        println!("creating hash clock");
        let mut hash_clock: HashClock = HashClock::new(genesis_hash_time, primary_validator.get_ticks_per_cycle());

        // fund second validator
        let signed_transaction: TransactionRequest<TransactionVariant> = payment_transaction(
            primary_pub_key,
            primary_validator.get_validator_private_key(),
            secondary_validator.get_validator_pub_key(),
            PRIMARY_ASSET_ID,
            SECONDARY_SEED_PAYMENT,
        )
        .unwrap();
        route_transaction(&mut primary_validator, &signed_transaction).unwrap();
        transactions.push(signed_transaction);

        let secondary_pub_key: AccountPubKey = secondary_validator.get_validator_pub_key();
        let secondary_balance: u64 = primary_validator
            .get_bank_controller()
            .get_balance(&secondary_pub_key, PRIMARY_ASSET_ID)
            .unwrap();
        assert!(
            secondary_balance == SECONDARY_SEED_PAYMENT,
            "Unexpected balance after funding second validator"
        );

        // stake second validator
        let signed_transaction: TransactionRequest<TransactionVariant> = stake_transaction(
            secondary_pub_key,
            secondary_validator.get_validator_private_key(),
            SECONDARY_SEED_PAYMENT,
        )
        .unwrap();
        route_transaction(&mut primary_validator, &signed_transaction).unwrap();
        transactions.push(signed_transaction);
        let secondary_staked: u64 = primary_validator
            .stake_controller
            .get_staked(&secondary_validator.get_validator_pub_key())
            .unwrap();
        assert!(
            secondary_staked == SECONDARY_SEED_PAYMENT,
            "Unexpected stake after staking secondary validator"
        );

        // process second block
        hash_clock.cycle();
        let second_block: Block<TransactionVariant> = primary_validator
            .propose_block(transactions, hash_clock.get_hash_time())
            .unwrap();
        // second validator does not need to vote as staked amount remains significantly less than primary
        primary_validator
            .validate_and_store_block(second_block, genesis_hash_time)
            .unwrap();

        // begin third block - here a second asset will be made and an orderbook inistantiated
        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();
        let signed_transaction: TransactionRequest<TransactionVariant> =
            asset_creation_transaction(secondary_pub_key, secondary_validator.get_validator_private_key()).unwrap();
        route_transaction(&mut primary_validator, &signed_transaction).unwrap();
        transactions.push(signed_transaction);

        let new_asset_balance: u64 = primary_validator
            .get_bank_controller()
            .get_balance(&secondary_pub_key, QUOTE_ASSET_ID)
            .unwrap();
        assert!(
            new_asset_balance == CREATED_ASSET_BALANCE,
            "Unexpected balance after second token genesis"
        );

        // TODO - add order book logic here
        let signed_transaction: TransactionRequest<TransactionVariant> = orderbook_creation_transaction(
            secondary_pub_key,
            secondary_validator.get_validator_private_key(),
            PRIMARY_ASSET_ID,
            QUOTE_ASSET_ID,
        )
        .unwrap();
        route_transaction(&mut primary_validator, &signed_transaction).unwrap();
        transactions.push(signed_transaction);

        let signed_transaction: TransactionRequest<TransactionVariant> = order_transaction(
            primary_pub_key,
            primary_validator.get_validator_private_key(),
            PRIMARY_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Ask,
            10,
            10,
        )
        .unwrap();
        route_transaction(&mut primary_validator, &signed_transaction).unwrap();
        transactions.push(signed_transaction);
        // TODO - play around w/ consensus to test it in more scenarios
    }
}
