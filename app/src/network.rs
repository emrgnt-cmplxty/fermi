#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use types::account::AccountPubKey;

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

        let mut i_step = 0;
        let mut next_block = second_block.clone();

        // finalize second block
        validator_one
            .validate_and_store_block(second_block, genesis_block)
            .unwrap();

        let mut validator_map: HashMap<AccountPubKey, u64> = HashMap::new();

        while i_step < 25 {
            let prev_block = next_block.clone();
            let next_validator = get_next_validator(&mut validator_one);

            validator_one.initialize_next_block();

            // begin block w/ dummy zero value transaction
            fund_and_stake_validator(&mut validator_one, &validator_two, 0);
            next_block = validator_one
                .propose_block(get_next_block_hash_time(
                    next_block.get_hash_time(),
                    next_block.get_block_hash(),
                ))
                .unwrap();

            validator_one.process_external_vote(&validator_two, &mut next_block, true);
            validator_one.process_external_vote(&validator_three, &mut next_block, true);
            // finalize second block
            validator_one
                .validate_and_store_block(next_block.clone(), prev_block)
                .unwrap();
            let count = validator_map.get(&next_validator).unwrap_or(&0);
            validator_map.insert(next_validator, count + 1);
            i_step += 1;
        }
        let mut i_count = 0;
        for (_addr, _count) in &validator_map {
            i_count += 1
        }
        // this test is probabilistic and should pass w/ probability ~1 - 1/3^20
        assert!(i_count == 3, "Failed to loop over 3 unique validators")
    }
}
