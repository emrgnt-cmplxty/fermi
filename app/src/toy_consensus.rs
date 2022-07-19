//! 
//! TODO
//! 0.) CREATE LOGICAL HANDLING FOR SIGNING & PROCESSING THIS SEED TOKEN TXN
//! 1.) BUILD TXN PROCESSING LOGIC WHICH EXECUTES CODE BELOW BY PROCESSING CREATED TXN ABOVE 
//! 2.) BUILD TXN CREATION & PROCESSING LOGIC WHICH STORES AND REPLICATES THIS LOGIC
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
        Payment, 
        TxnVariant::PaymentTransaction,
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
    bank::{BankController, BANK_ACCOUNT_BYTES, STAKE_ASSET_ID},
    stake::{StakeController},
};
use types::{
    account::{AccountPubKey, AccountPrivKey, AccountSignature},
    spot::{DiemCryptoMessage},
};

// Specify # of tokens created at genesis
const GENESIS_SEED_PAYMENT: u64 = 1_000_000;
// Specify # of tokens sent to second validator
const VALIDATOR_SEED_PAYMENT: u64 = 100_000;
pub struct ConsensusManager
{
    block_container: BlockContainer<TxnVariant>,
    hash_clock: HashClock,
    bank_controller: BankController,
    stake_controller: StakeController,
    validator_pub_key: AccountPubKey,
    validator_private_key: AccountPrivKey,
}

// TOY CONSENSUS
pub fn genesis_token_txn(validator_pub_key: AccountPubKey, validator_private_key: &AccountPrivKey) -> TxnRequest<TxnVariant> {
    let bank_pub_key: AccountPubKey = AccountPubKey::from_bytes_unchecked(&BANK_ACCOUNT_BYTES).unwrap();

    let txn: TxnVariant = PaymentTransaction(
        Payment::new(
            bank_pub_key,
            validator_pub_key,
            STAKE_ASSET_ID,
            GENESIS_SEED_PAYMENT
        )
    );
    let txn_hash: HashValue = txn.hash();
    // TODO #0 //
    let signed_hash: AccountSignature  = (*validator_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    // NOTE, THIS SPECIAL TRANSACATON WOULD FAIL VERIFICATION BECAUSE SIGNING IS INCORRECT 
    TxnRequest::<TxnVariant>::new(
        txn,
        validator_pub_key, 
        signed_hash 
    )
}
pub fn get_genesis_block(validator_pub_key: &AccountPubKey, validator_private_key: &AccountPrivKey, bank_controller: &mut BankController, stake_controller: &mut StakeController, hash_clock: &HashClock) -> Block<TxnVariant> {
    let mut txns: Vec<TxnRequest<TxnVariant>> = Vec::new();

    // GENESIS TXN #0
    let signed_txn: TxnRequest<TxnVariant> = genesis_token_txn(*validator_pub_key, validator_private_key);
    txns.push(signed_txn);
    // TODO # 1 //
    bank_controller.create_account(&validator_pub_key).unwrap();
    let bank_pub_key: AccountPubKey = AccountPubKey::from_bytes_unchecked(&BANK_ACCOUNT_BYTES).unwrap();
    bank_controller.transfer(&bank_pub_key, validator_pub_key, STAKE_ASSET_ID, GENESIS_SEED_PAYMENT);

    // GENESIS TXN 1
    // TODO # 2 //
    stake_controller.create_account(&validator_pub_key).unwrap();
    stake_controller.stake(bank_controller, &validator_pub_key, ((GENESIS_SEED_PAYMENT - VALIDATOR_SEED_PAYMENT) as i64).try_into().unwrap());

    // RETURN FIRST BLACK
    let block_hash: HashValue = generate_block_hash(&txns);
    let dummy_vote_cert: VoteCert = VoteCert::new(0);
    Block::<TxnVariant>::new(txns, *validator_pub_key, block_hash, hash_clock.get_time(), dummy_vote_cert)
}

// pub fn get_vote_cert(stake_controller: StakeController) {

// }

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

    pub fn init_new_consensus(&mut self) {
        let genesis_block = get_genesis_block(&self.validator_pub_key, &self.validator_private_key, &mut self.bank_controller, &mut self.stake_controller, &self.hash_clock);
        self.block_container.append_block(genesis_block);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_consensus() {
        let mut primary_validator: ConsensusManager = ConsensusManager::new();
        // let mut secondary_validator: ConsensusManager = ConsensusManager::new();
        primary_validator.init_new_consensus();

    }
}