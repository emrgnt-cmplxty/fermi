#[cfg(test)]
mod tests {

    use super::super::validator::{
        tests::fund_and_stake_validator, tests::get_next_block_hash_time, ValidatorController, GENESIS_STAKE_AMOUNT,
    };

    use types::asset::AssetId;
    const QUOTE_ASSET_ID: AssetId = 1;
    const SECONDARY_SEED_PAYMENT: u64 = (0.33 * GENESIS_STAKE_AMOUNT as f64) as u64;

    #[test]
    fn test_network() {
        let mut validator_one = ValidatorController::new();
        // initialize
        let genesis_block = validator_one.build_genesis_block().unwrap();
        let validator_two = ValidatorController::new();
        let validator_three = ValidatorController::new();
        fund_and_stake_validator(&mut validator_one, &validator_two, SECONDARY_SEED_PAYMENT);
        fund_and_stake_validator(&mut validator_one, &validator_three, SECONDARY_SEED_PAYMENT);

        let block_transactions = validator_one.get_latest_transactions().clone();
        let block_hash = get_next_block_hash_time(genesis_block.get_hash_time(), genesis_block.get_block_hash());
        let _block_proposal = validator_one.propose_block(block_transactions, block_hash).unwrap();
    }
}
