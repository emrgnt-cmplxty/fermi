use proc::stake::StakeController;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::convert::TryInto;
use types::account::AccountPubKey;

use crate::validator::ValidatorController;

const THRESHOLD_FOR_LEADER_ELECTION: f64 = 0.01;

pub fn get_valid_validators(stake_controller: &StakeController) -> Vec<AccountPubKey> {
    let mut valid_validators: Vec<AccountPubKey> = Vec::new();
    for (address, account) in stake_controller.get_accounts() {
        if (account.get_staked_amount() as f64 / stake_controller.get_total_staked() as f64)
            > THRESHOLD_FOR_LEADER_ELECTION
        {
            valid_validators.push(*address);
        }
    }
    valid_validators
}

pub fn get_next_validator(validator: &mut ValidatorController) -> AccountPubKey {
    let valid_validators = get_valid_validators(validator.get_stake_controller());
    let blocks = validator.get_block_container().get_blocks();
    let last_block_hash = blocks[blocks.len() - 1].get_block_hash();
    let mut rng: StdRng = SeedableRng::from_seed(last_block_hash.to_vec().try_into().unwrap());
    valid_validators[rng.gen_range(0, valid_validators.len())]
}
