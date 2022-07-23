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
    account::{AccountPrivKey, AccountPubKey},
    error::GDEXError,
    hash_clock::HashTime,
};

// specify the number of tokens creator stakes during genesis
pub const GENESIS_STAKE_AMOUNT: u64 = 1_000_000;

// the consensus manager owns all Controllers and is responsible for
// processing transactions, updating state, and reaching consensus in "toy" conditions
pub struct ValidatorController {
    pending_block: Option<Block<TransactionVariant>>,
    block_container: BlockContainer<TransactionVariant>,
    bank_controller: BankController,
    spot_controller: SpotController,
    stake_controller: StakeController,
    ticks_per_cycle: u64,
    pub_key: AccountPubKey,
    private_key: AccountPrivKey,
}
impl ValidatorController {
    pub fn new() -> Self {
        let (pub_key, private_key) = generate_key_pair();
        ValidatorController {
            pending_block: None,
            block_container: BlockContainer::new(),
            bank_controller: BankController::new(),
            spot_controller: SpotController::new(),
            stake_controller: StakeController::new(),
            ticks_per_cycle: DEFAULT_TICKS_PER_CYCLE,
            pub_key,
            private_key,
        }
    }

    // append transaction to latest block transactions
    // note that routåe transaction gaurentees successful
    // signature verification and transaction execution
    fn process_and_store_transaction(
        &mut self,
        signed_transaction: TransactionRequest<TransactionVariant>,
    ) -> Result<(), GDEXError> {
        route_transaction(self, &signed_transaction)?;
        match &mut self.pending_block {
            Some(block) => {
                block.push_transaction(signed_transaction);
                Ok(())
            }
            None => Err(GDEXError::PendingBlock(
                "This validator has no pending block".to_string(),
            )),
        }
    }

    // this function creates and internalizes the genesis block of a new chain
    // it begins by creating primary asset of the blockchain which defaults ownership
    // to the local validator, it then stakes some balance from the validators perspective
    // staking of funds allows consensus to formally begin, however it should be defined
    pub fn build_genesis_block(&mut self) -> Result<Block<TransactionVariant>, GDEXError> {
        // initialize a block for later proposal
        self.initialize_next_block();
        // GENESIS transaction #0 -> create the base asset of the blockhain
        // <-- there is some hair on this process as we currently issue all tokens at genesis to the initial generator -->
        // --> however, this can be alleviated by extending the functionality of the bank module <--
        let genesis_token_transaction = asset_creation_transaction(self.pub_key, &self.private_key)?;
        self.process_and_store_transaction(genesis_token_transaction)?;
        // GENESIS transaction 1 -> stake some manager funds to allow consensus to begin
        let genesis_stake_transaction = stake_transaction(self.pub_key, &self.private_key, GENESIS_STAKE_AMOUNT)?;
        self.process_and_store_transaction(genesis_stake_transaction)?;

        // create and synchronously tick the initial hash clock
        let mut hash_clock = HashClock::default();
        hash_clock.cycle();

        // return the initial genesis block
        self.propose_block(hash_clock.get_hash_time())
    }

    // initialized blocks have an incorrect block hash and an incomplete vote certficiate
    // the initialized block has a dummy hash time initially
    // this likely should be replaced by an Option at some point in the new future
    pub fn initialize_next_block(&mut self) {
        let transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();
        let block_hash = generate_block_hash(&transactions);
        let vote_cert = VoteCert::new(self.stake_controller.get_total_staked(), block_hash);
        self.pending_block = Some(Block::<TransactionVariant>::new(
            transactions,
            self.pub_key,
            block_hash,
            self.block_container.get_blocks().len() as u64,
            block_hash, // append a dummy hash time, TODO - change to option to remove need of this
            vote_cert,
        ));
    }

    // this function assumes a successfully initialized block
    // proposal is done from the perspective of the validator and includes
    // the validators vote as well as the included proposed block
    // lastly, the validator stores this into state as the latest pending block
    pub fn propose_block(&mut self, proposed_hash_time: HashTime) -> Result<Block<TransactionVariant>, GDEXError> {
        match &mut self.pending_block {
            Some(pending_block) => {
                let vote_cert = VoteCert::new(self.stake_controller.get_total_staked(), pending_block.get_block_hash());
                // validator signs the block hash appended to their vote response
                let validator_signature = self.private_key.sign(&vote_cert.compute_vote_msg(true));
                pending_block.append_vote(
                    self.pub_key,
                    validator_signature,
                    true,
                    self.stake_controller.get_staked(&self.pub_key)?,
                )?;

                pending_block.update_hash_time(proposed_hash_time);

                Ok(pending_block.clone())
            }
            None => Err(GDEXError::PendingBlock(
                "This validator has no pending block".to_string(),
            )),
        }
    }

    // this method allows unprotected saving of a block
    // unprocted saving of a block should only occur at genesis
    pub fn store_genesis_block(&mut self, block: Block<TransactionVariant>) {
        // save block
        self.block_container.append_block(block);
        // erase the pending block
        self.pending_block = None;
    }

    // this method validates a propsed block and stores into validator state
    // the validation of a pending block is still quite nascent in the code below
    pub fn validate_and_store_block(
        &mut self,
        pending_block: Block<TransactionVariant>,
        prev_block: Block<TransactionVariant>,
    ) -> Result<(), GDEXError> {
        let mut hash_clock = HashClock::default();
        // mix hash time with trailing block
        hash_clock.update_hash_time(prev_block.get_hash_time(), prev_block.get_block_hash());
        hash_clock.cycle();

        if pending_block.get_vote_cert().vote_has_passed()
            && pending_block.get_hash_time() == hash_clock.get_hash_time()
            && pending_block.get_block_number() == prev_block.get_block_number() + 1
        {
            // save block
            self.block_container.append_block(pending_block);
            // overwrite latest block transactions
            self.pending_block = None;
            Ok(())
        } else {
            Err(GDEXError::BlockValidation("Validation failed".to_string()))
        }
    }

    pub fn process_external_vote(
        &mut self,
        external_validator: &ValidatorController,
        pending_block: &mut Block<TransactionVariant>,
        vote_response: bool,
    ) {
        let external_validator_pub_key = external_validator.get_pub_key();
        let external_validator_signature = external_validator
            .get_private_key()
            .sign(&pending_block.get_vote_cert().compute_vote_msg(true));
        let validator_stake = self.stake_controller.get_staked(&external_validator_pub_key).unwrap();

        pending_block
            .append_vote(
                external_validator_pub_key,
                external_validator_signature,
                vote_response,
                validator_stake,
            )
            .unwrap();
    }

    pub fn get_latest_transactions(&self) -> Result<&Vec<TransactionRequest<TransactionVariant>>, GDEXError> {
        match &self.pending_block {
            Some(pending_block) => Ok(pending_block.get_transactions()),
            None => Err(GDEXError::PendingBlock(
                "This validator has no pending block".to_string(),
            )),
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

impl Default for ValidatorController {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_next_block_hash_time(prev_block_hash_time: HashTime, prev_block_hash: HashValue) -> HashTime {
    let mut hash_clock = HashClock::default();
    hash_clock.update_hash_time(prev_block_hash_time, prev_block_hash);
    hash_clock.cycle();
    hash_clock.get_hash_time()
}
#[cfg(test)]
pub mod tests {
    use super::super::router::{order_transaction, orderbook_creation_transaction, payment_transaction};
    use super::*;
    use proc::bank::{CREATED_ASSET_BALANCE, PRIMARY_ASSET_ID};
    use types::{asset::AssetId, orderbook::OrderSide};
    const QUOTE_ASSET_ID: AssetId = 1;

    pub fn fund_and_stake_validator(
        validator_a: &mut ValidatorController,
        validator_b: &ValidatorController,
        amount: u64,
    ) {
        // fund and stake second validator
        let signed_transaction = payment_transaction(
            validator_a.get_pub_key(),
            validator_a.get_private_key(),
            validator_b.get_pub_key(),
            PRIMARY_ASSET_ID,
            amount,
        )
        .unwrap();
        validator_a.process_and_store_transaction(signed_transaction).unwrap();
        let signed_transaction =
            stake_transaction(validator_b.get_pub_key(), validator_b.get_private_key(), amount).unwrap();
        validator_a.process_and_store_transaction(signed_transaction).unwrap();
    }

    #[test]
    fn test_build_genesis_block() {
        // start w/ a two-validator setup
        let mut primary_validator = ValidatorController::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block = primary_validator.build_genesis_block().unwrap();

        // check genesis block has expected number of transactions before storing
        assert!(
            genesis_block.get_transactions().len() == 2,
            "Wrong block transaction length after creating genesis block"
        );
        // validate block immediately as genesis proposer is only staked validator
        primary_validator.store_genesis_block(genesis_block);

        // check validator has no pending block after storing the genesis block
        assert!(matches!(
            primary_validator.get_latest_transactions(),
            Err(GDEXError::PendingBlock(_))
        ));

        // check that the initial funding was successful by checking state of controllers
        let primary_pub_key = primary_validator.get_pub_key();
        let primary_balance = primary_validator
            .get_bank_controller()
            .get_balance(&primary_pub_key, PRIMARY_ASSET_ID)
            .unwrap();
        assert!(
            primary_balance == CREATED_ASSET_BALANCE - GENESIS_STAKE_AMOUNT,
            "Unexpected balance after token genesis & stake"
        );
        let primary_staked = primary_validator
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
        let mut primary_validator = ValidatorController::new();
        let secondary_validator = ValidatorController::new();

        // repeat genesis setup
        let genesis_block = primary_validator.build_genesis_block().unwrap();
        primary_validator.store_genesis_block(genesis_block.clone());

        // begin second block where second validator is funded and staked
        primary_validator.initialize_next_block();

        // fund and stake second validator
        fund_and_stake_validator(&mut primary_validator, &secondary_validator, SECONDARY_SEED_PAYMENT);

        let block_hash_time = get_next_block_hash_time(genesis_block.get_hash_time(), genesis_block.get_block_hash());
        let second_block = primary_validator.propose_block(block_hash_time).unwrap();

        // second validator does not need to vote as staked amount remains significantly less than primary
        primary_validator
            .validate_and_store_block(second_block, genesis_block)
            .unwrap();

        // check validator has no pending block after validating the second block
        assert!(matches!(
            primary_validator.get_latest_transactions(),
            Err(GDEXError::PendingBlock(_))
        ));

        let secondary_balance = primary_validator
            .get_bank_controller()
            .get_balance(&secondary_validator.get_pub_key(), PRIMARY_ASSET_ID)
            .unwrap();

        let secondary_staked = primary_validator
            .get_stake_controller()
            .get_staked(&secondary_validator.get_pub_key())
            .unwrap();
        assert!(
            secondary_balance == 0,
            "Unexpected balance after staking second validator"
        );

        assert!(
            secondary_staked == SECONDARY_SEED_PAYMENT,
            "Unexpected stake after staking second validator"
        );
    }

    #[test]
    #[should_panic]
    fn test_two_validator_one_vote_fails() {
        const SECONDARY_SEED_PAYMENT: u64 = (0.5 * (CREATED_ASSET_BALANCE as f64)) as u64;
        let mut primary_validator = ValidatorController::new();
        let secondary_validator = ValidatorController::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block = primary_validator.build_genesis_block().unwrap();
        primary_validator.store_genesis_block(genesis_block.clone());

        // begin second block where second validator is funded and staked
        primary_validator.initialize_next_block();

        fund_and_stake_validator(&mut primary_validator, &secondary_validator, SECONDARY_SEED_PAYMENT);
        let second_block = primary_validator
            .propose_block(get_next_block_hash_time(
                genesis_block.get_hash_time(),
                genesis_block.get_block_hash(),
            ))
            .unwrap();

        // second block should pass since staked amount is computed at beginning of block
        primary_validator
            .validate_and_store_block(second_block.clone(), genesis_block)
            .unwrap();

        // begin third block where failure should occur
        primary_validator.initialize_next_block();

        // 0 value payment transaction to test for failure
        fund_and_stake_validator(&mut primary_validator, &secondary_validator, 0);
        let third_block = primary_validator
            .propose_block(get_next_block_hash_time(
                second_block.get_hash_time(),
                second_block.get_block_hash(),
            ))
            .unwrap();

        // third block should now fail since second validator stake is accounted for and not voting
        primary_validator
            .validate_and_store_block(third_block, second_block)
            .unwrap();
    }

    #[test]
    fn test_two_validator_two_votes() {
        const SECONDARY_SEED_PAYMENT: u64 = (0.5 * (CREATED_ASSET_BALANCE as f64)) as u64;
        let mut primary_validator = ValidatorController::new();
        let secondary_validator = ValidatorController::new();

        // initiate new consensus by creating the genesis block from perspective of primary validator
        let genesis_block = primary_validator.build_genesis_block().unwrap();
        primary_validator.store_genesis_block(genesis_block.clone());

        // begin second block where second validator is funded and staked
        primary_validator.initialize_next_block();

        fund_and_stake_validator(&mut primary_validator, &secondary_validator, SECONDARY_SEED_PAYMENT);
        let mut second_block = primary_validator
            .propose_block(get_next_block_hash_time(
                genesis_block.get_hash_time(),
                genesis_block.get_block_hash(),
            ))
            .unwrap();

        primary_validator.process_external_vote(&secondary_validator, &mut second_block, true);

        // consensus will now pass since second validator has cast an affirmative vote
        primary_validator
            .validate_and_store_block(second_block.clone(), genesis_block)
            .unwrap();
    }

    #[test]
    fn test_orderbook_workflow() {
        let mut primary_validator = ValidatorController::new();
        // initialize
        primary_validator.build_genesis_block().unwrap();

        let signed_transaction =
            asset_creation_transaction(primary_validator.get_pub_key(), primary_validator.get_private_key()).unwrap();
        primary_validator
            .process_and_store_transaction(signed_transaction)
            .unwrap();

        let signed_transaction = orderbook_creation_transaction(
            primary_validator.get_pub_key(),
            primary_validator.get_private_key(),
            PRIMARY_ASSET_ID,
            QUOTE_ASSET_ID,
        )
        .unwrap();
        primary_validator
            .process_and_store_transaction(signed_transaction)
            .unwrap();
        let signed_transaction = order_transaction(
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
