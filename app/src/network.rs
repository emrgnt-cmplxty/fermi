#[cfg(test)]
// extern crate core;
// use core::transaction::TransactionRequest;
// use core::transaction::TransactionVariant;

// fn propagate_message(_transaction: Vec<TransactionRequest<TransactionVariant>>) {}

#[cfg(test)]
mod tests {
    use super::super::validator::{ValidatorController, GENESIS_STAKE_AMOUNT};
    use super::super::router::{payment_transaction};
    use proc::bank::{PRIMARY_ASSET_ID};

    const SECONDARY_SEED_PAYMENT: u64 = (0.33 * GENESIS_STAKE_AMOUNT as f64) as u64;

    #[test]
    fn test_network() {
        let mut validator_one = ValidatorController::new();
        // initialize
        let validator_two = ValidatorController::new();
        let validator_three = ValidatorController::new();
        let genesis_block = validator_one.build_genesis_block().unwrap();

        // fund validator two
        let fund_two_transaction = payment_transaction(
            validator_one.get_pub_key(),
            validator_one.get_private_key(),
            validator_two.get_pub_key(),
            PRIMARY_ASSET_ID,
            SECONDARY_SEED_PAYMENT,
        )
        .unwrap();

        // fund validator two
        let fund_three_transaction = payment_transaction(
            validator_one.get_pub_key(),
            validator_one.get_private_key(),
            validator_three.get_pub_key(),
            PRIMARY_ASSET_ID,
            SECONDARY_SEED_PAYMENT,
        )
        .unwrap();

        let validators = vec![validator_one, validator_two, validator_three];

        for mut validator in validators {
            validator.store_genesis_block(genesis_block.clone());
        }

        let _transactions = vec![fund_two_transaction, fund_three_transaction];

        // let block_transactions = validator_one.get_latest_transactions().clone();
        // let block_hash = get_next_block_hash_time(genesis_block.get_hash_time(), genesis_block.get_block_hash());
        // let _block_proposal = validator_one.propose_block(block_transactions, block_hash).unwrap();
    }
}
