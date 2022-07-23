#[cfg(test)]
mod tests {
    use super::super::rotation::get_next_validator;
    use super::super::validator::{
        get_next_block_hash_time, tests::fund_and_stake_validator, ValidatorController, GENESIS_STAKE_AMOUNT,
    };

    const SECONDARY_SEED_PAYMENT: u64 = (0.33 * GENESIS_STAKE_AMOUNT as f64) as u64;

    #[test]
    fn test_validator_rotation() {
        let mut validator_one = ValidatorController::new();
        let mut validator_two = ValidatorController::new();
        let mut validator_three = ValidatorController::new();
        let genesis_block = validator_one.build_genesis_block().unwrap();

        validator_one.store_genesis_block(genesis_block.clone());
        validator_two.store_genesis_block(genesis_block.clone());
        validator_three.store_genesis_block(genesis_block.clone());

        // get next validator after finalizing block
        let next_validator = get_next_validator(&mut validator_one);
        assert!(next_validator == validator_one.get_pub_key());

        // begin second block where other validators are funded and staked
        validator_one.initialize_next_block();

        // begin second block where second validator is funded and staked
        fund_and_stake_validator(&mut validator_one, &validator_two, SECONDARY_SEED_PAYMENT);
        fund_and_stake_validator(&mut validator_one, &validator_three, SECONDARY_SEED_PAYMENT);

        let second_block = validator_one
            .propose_block(get_next_block_hash_time(
                genesis_block.get_hash_time(),
                genesis_block.get_block_hash(),
            ))
            .unwrap();

        // finalize second block
        validator_one
            .validate_and_store_block(second_block, genesis_block)
            .unwrap();
    }
}
