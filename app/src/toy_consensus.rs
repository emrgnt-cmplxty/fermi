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
    hash_clock::{HashClock, DEFAULT_TICKS_PER_CYCLE},
    transaction::{TransactionRequest, TransactionVariant},
    vote_cert::VoteCert,
};
use gdex_crypto::{hash::HashValue, SigningKey};
use proc::{account::generate_key_pair, bank::BankController, spot::SpotController, stake::StakeController};
use types::{
    account::{AccountPrivKey, AccountPubKey, AccountSignature},
    error::GDEXError,
    hash_clock::HashTime,
};

// specify the number of tokens creator stakes during genesis
const GENESIS_STAKE_AMOUNT: u64 = 1_000_000;

// the consensus manager owns all Controllers and is responsible for
// processing transactions, updating state, and reaching consensus in "toy" conditions
pub struct ConsensusManager {
    latest_block_transactions: Vec<TransactionRequest<TransactionVariant>>,
    block_container: BlockContainer<TransactionVariant>,
    bank_controller: BankController,
    spot_controller: SpotController,
    stake_controller: StakeController,
    ticks_per_cycle: u64,
    pub_key: AccountPubKey,
    private_key: AccountPrivKey,
}
impl ConsensusManager {
    pub fn new() -> Self {
        let (pub_key, private_key) = generate_key_pair();
        ConsensusManager {
            latest_block_transactions: Vec::new(),
            block_container: BlockContainer::new(),
            bank_controller: BankController::new(),
            spot_controller: SpotController::new(),
            stake_controller: StakeController::new(),
            ticks_per_cycle: DEFAULT_TICKS_PER_CYCLE,
            pub_key,
            private_key,
        }
    }

    // append transaction to latest block transactions after successful verification
    fn process_and_store_transaction(
        &mut self,
        signed_transaction: TransactionRequest<TransactionVariant>,
    ) -> Result<(), GDEXError> {
        route_transaction(self, &signed_transaction)?;
        self.latest_block_transactions.push(signed_transaction);
        Ok(())
    }

    // build the genesis block by creating the base asset and staking some funds
    pub fn build_genesis_block(&mut self) -> Result<Block<TransactionVariant>, GDEXError> {
        // create and synchronously tick the initial hash clock
        let mut hash_clock: HashClock = HashClock::default();
        hash_clock.cycle();

        // GENESIS transaction #0 -> create the base asset of the blockhain
        // <-- there is some hair on this process as we currently issue all tokens at genesis to the initial generator -->
        // --> however, this can be alleviated by extending the functionality of the bank module <--
        self.process_and_store_transaction(asset_creation_transaction(self.pub_key, &self.private_key)?)?;

        // GENESIS transaction 1 -> stake some manager funds to allow consensus to begin
        self.process_and_store_transaction(stake_transaction(
            self.pub_key,
            &self.private_key,
            GENESIS_STAKE_AMOUNT,
        )?)?;

        // return the initial genesis block
        self.propose_block(self.latest_block_transactions.clone(), hash_clock.get_hash_time())
    }

    // take a list of transactions and create a new block w/ the managers vote included
    pub fn propose_block(
        &self,
        transactions: Vec<TransactionRequest<TransactionVariant>>,
        block_hash_time: HashTime,
    ) -> Result<Block<TransactionVariant>, GDEXError> {
        let block_hash: HashValue = generate_block_hash(&transactions);
        let mut vote_cert: VoteCert = VoteCert::new(self.stake_controller.get_total_staked(), block_hash);

        // validator signs the block hash appended to their vote response
        let validator_signature: AccountSignature = self.private_key.sign(&vote_cert.compute_vote_msg(true));

        vote_cert.append_vote(
            self.pub_key,
            validator_signature,
            true,
            self.stake_controller.get_staked(&self.pub_key)?,
        )?;

        Ok(Block::<TransactionVariant>::new(
            transactions,
            self.pub_key,
            block_hash,
            self.block_container.get_blocks().len() as u64,
            block_hash_time,
            vote_cert,
        ))
    }

    pub fn store_genesis_block(&mut self, block: Block<TransactionVariant>) {
        // save block
        self.block_container.append_block(block);
        // overwrite latest block transactions
        self.latest_block_transactions = Vec::new();
    }

    pub fn validate_and_store_block(
        &mut self,
        block: Block<TransactionVariant>,
        prev_block: Block<TransactionVariant>,
    ) -> Result<(), GDEXError> {
        let mut hash_clock: HashClock = HashClock::default();
        // mix hash time with trailing block
        hash_clock.update_hash_time(prev_block.get_hash_time(), prev_block.get_block_hash());
        hash_clock.cycle();

        if block.get_vote_cert().vote_has_passed()
            && block.get_hash_time() == hash_clock.get_hash_time()
            && block.get_block_number() == prev_block.get_block_number() + 1
        {
            // save block
            self.block_container.append_block(block);
            // overwrite latest block transactions
            self.latest_block_transactions = Vec::new();
            Ok(())
        } else {
            Err(GDEXError::BlockValidation("Validation failed".to_string()))
        }
    }

    pub fn get_latest_transactions(&self) -> &Vec<TransactionRequest<TransactionVariant>> {
        &self.latest_block_transactions
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

    pub fn get_pub_key(&self) -> AccountPubKey {
        self.pub_key
    }

    pub fn get_private_key(&self) -> &AccountPrivKey {
        &self.private_key
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
    use proc::bank::{CREATED_ASSET_BALANCE, PRIMARY_ASSET_ID};
    use types::{asset::AssetId, orderbook::OrderSide};
    const QUOTE_ASSET_ID: AssetId = 1;

    #[test]
    fn test_build_genesis_block() {
        // start w/ a two-validator setup
        let mut primary_validator: ConsensusManager = ConsensusManager::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block: Block<TransactionVariant> = primary_validator.build_genesis_block().unwrap();

        // check genesis block has expected number of transactions before storing
        assert!(
            genesis_block.get_transactions().len() == 2,
            "Wrong block transaction length after creating genesis block"
        );
        // validate block immediately as genesis proposer is only staked validator
        primary_validator.store_genesis_block(genesis_block.clone());

        // check validator has clean transaction slate after processing genesis block
        assert!(
            primary_validator.get_latest_transactions().len() == 0,
            "Wrong transaction length after processing genesis block"
        );

        // check that the initial funding was successful by checking state of controllers
        let primary_pub_key: AccountPubKey = primary_validator.get_pub_key();
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
    }

    #[test]
    fn test_two_validator_one_vote() {
        const SECONDARY_SEED_PAYMENT: u64 = 1_000;
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let secondary_validator: ConsensusManager = ConsensusManager::new();

        // repeat genesis setup
        let genesis_block: Block<TransactionVariant> = primary_validator.build_genesis_block().unwrap();
        primary_validator.store_genesis_block(genesis_block.clone());

        // begin second block where second validator is funded and staked

        // initialize and cycle hashclock
        let mut hash_clock: HashClock = HashClock::default();
        hash_clock.update_hash_time(genesis_block.get_hash_time(), genesis_block.get_block_hash());
        hash_clock.cycle();

        // fund and stake second validator
        fund_and_stake_validator(&mut primary_validator, &secondary_validator, SECONDARY_SEED_PAYMENT);

        let second_block: Block<TransactionVariant> = primary_validator
            .propose_block(
                primary_validator.get_latest_transactions().clone(),
                hash_clock.get_hash_time(),
            )
            .unwrap();

        // second validator does not need to vote as staked amount remains significantly less than primary
        primary_validator
            .validate_and_store_block(second_block.clone(), genesis_block)
            .unwrap();

        // check that block has been stored and transactions whipted
        assert!(
            primary_validator.get_latest_transactions().len() == 0,
            "Wrong transaction length after processing second block"
        );
    }

    fn fund_and_stake_validator(validator_a: &mut ConsensusManager, validator_b: &ConsensusManager, amount: u64) {
        // fund and stake second validator
        let signed_transaction: TransactionRequest<TransactionVariant> = payment_transaction(
            validator_a.get_pub_key(),
            validator_a.get_private_key(),
            validator_b.get_pub_key(),
            PRIMARY_ASSET_ID,
            amount,
        )
        .unwrap();
        validator_a.process_and_store_transaction(signed_transaction).unwrap();
        let signed_transaction: TransactionRequest<TransactionVariant> =
            stake_transaction(validator_b.get_pub_key(), validator_b.get_private_key(), amount).unwrap();
        validator_a.process_and_store_transaction(signed_transaction).unwrap();
    }

    fn get_next_block_hash_time(prev_block_hash_time: HashTime, prev_block_hash: HashValue) -> HashTime {
        let mut hash_clock: HashClock = HashClock::default();
        hash_clock.update_hash_time(prev_block_hash_time, prev_block_hash);
        hash_clock.cycle();
        hash_clock.get_hash_time()
    }

    #[test]
    #[should_panic]
    fn test_two_validator_one_vote_fails() {
        const SECONDARY_SEED_PAYMENT: u64 = (0.5 * (CREATED_ASSET_BALANCE as f64)) as u64;
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let secondary_validator: ConsensusManager = ConsensusManager::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block: Block<TransactionVariant> = primary_validator.build_genesis_block().unwrap();
        primary_validator.store_genesis_block(genesis_block.clone());

        // begin second block where second validator is funded and staked
        fund_and_stake_validator(&mut primary_validator, &secondary_validator, SECONDARY_SEED_PAYMENT);

        let second_block: Block<TransactionVariant> = primary_validator
            .propose_block(
                primary_validator.get_latest_transactions().clone(),
                get_next_block_hash_time(genesis_block.get_hash_time(), genesis_block.get_block_hash()),
            )
            .unwrap();

        // validate block should fail since secondary validator has large share of staked
        primary_validator
            .validate_and_store_block(second_block.clone(), genesis_block)
            .unwrap();
    }

    #[test]
    fn test_two_validator_two_votes() {
        const SECONDARY_SEED_PAYMENT: u64 = (0.5 * (CREATED_ASSET_BALANCE as f64)) as u64;
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        let secondary_validator: ConsensusManager = ConsensusManager::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block: Block<TransactionVariant> = primary_validator.build_genesis_block().unwrap();
        primary_validator.store_genesis_block(genesis_block.clone());

        // begin second block where second validator is funded and staked
        fund_and_stake_validator(&mut primary_validator, &secondary_validator, SECONDARY_SEED_PAYMENT);

        let mut second_block: Block<TransactionVariant> = primary_validator
            .propose_block(
                primary_validator.get_latest_transactions().clone(),
                get_next_block_hash_time(genesis_block.get_hash_time(), genesis_block.get_block_hash()),
            )
            .unwrap();

        let validator_signature: AccountSignature = secondary_validator
            .get_private_key()
            .sign(&second_block.get_vote_cert().compute_vote_msg(true));

        second_block
            .append_vote(
                secondary_validator.get_pub_key(),
                validator_signature,
                true,
                primary_validator
                    .stake_controller
                    .get_staked(&secondary_validator.get_pub_key())
                    .unwrap(),
            )
            .unwrap();

        // consensus will now pass since second validator has cast an affirmative vote
        primary_validator
            .validate_and_store_block(second_block.clone(), genesis_block)
            .unwrap();
    }

    #[test]
    fn test_orderbook_workflow() {
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        // initialize
        primary_validator.build_genesis_block().unwrap();

        let signed_transaction: TransactionRequest<TransactionVariant> =
            asset_creation_transaction(primary_validator.get_pub_key(), primary_validator.get_private_key()).unwrap();
        primary_validator
            .process_and_store_transaction(signed_transaction)
            .unwrap();

        let signed_transaction: TransactionRequest<TransactionVariant> = orderbook_creation_transaction(
            primary_validator.get_pub_key(),
            primary_validator.get_private_key(),
            PRIMARY_ASSET_ID,
            QUOTE_ASSET_ID,
        )
        .unwrap();
        primary_validator
            .process_and_store_transaction(signed_transaction)
            .unwrap();
        let signed_transaction: TransactionRequest<TransactionVariant> = order_transaction(
            primary_validator.get_pub_key(),
            primary_validator.get_private_key(),
            PRIMARY_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Ask,
            10,
            10,
        )
        .unwrap();
        primary_validator
            .process_and_store_transaction(signed_transaction)
            .unwrap();
    }
}
