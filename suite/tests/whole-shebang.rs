//! THE WHOLE SHEBANG TEST IS AN ONGOING WORK IN PROGRESS
//! HERE WE WILL CONSTRUCT A LARGE-SCALE TEST THAT LOOKS
//! TO SHOW THAT THE ENTIRE CODEBASE IS GENERALLY SOUND
//!
mod tests {
    extern crate core;
    extern crate engine;

    use core::{
        block::{generate_block_hash, Block, BlockContainer},
        hash_clock::HashClock,
        transaction::{TransactionRequest, TransactionVariant},
        vote_cert::VoteCert,
    };
    use engine::orders::new_limit_order_request;
    use gdex_crypto::{
        hash::{CryptoHash, HashValue},
        SigningKey, Uniform,
    };
    use proc::{bank::BankController, spot::SpotController};
    use rand::rngs::ThreadRng;
    use std::time;
    use types::{
        account::{AccountPrivKey, AccountPubKey, AccountSignature},
        asset::AssetId,
        orderbook::OrderSide,
        spot::DiemCryptoMessage,
    };

    #[test]
    fn test_orderbook_signed_transaction() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();
        let base_asset_id: AssetId = bank_controller.create_asset(&account_pub_key).unwrap();
        let quote_asset_id: AssetId = bank_controller.create_asset(&account_pub_key).unwrap();

        let mut spot_controller: SpotController = SpotController::new();
        spot_controller.create_orderbook(base_asset_id, quote_asset_id).unwrap();

        let price: u64 = 1;
        let quantity: u64 = 10;
        let transaction: TransactionVariant = TransactionVariant::OrderTransaction(new_limit_order_request(
            base_asset_id,
            quote_asset_id,
            OrderSide::Bid,
            price,
            quantity,
            time::SystemTime::now(),
        ));

        let transaction_hash: HashValue = transaction.hash();
        let signed_hash: AccountSignature = private_key.sign(&DiemCryptoMessage(transaction_hash.to_string()));
        let signed_transaction: TransactionRequest<TransactionVariant> =
            TransactionRequest::<TransactionVariant>::new(transaction, account_pub_key, signed_hash);

        spot_controller
            .parse_limit_order_transaction(&mut bank_controller, &signed_transaction)
            .unwrap();
        signed_transaction.verify_transaction().unwrap();

        let mut transactions: Vec<TransactionRequest<TransactionVariant>> = Vec::new();
        transactions.push(signed_transaction);
        let block_hash: HashValue = generate_block_hash(&transactions);
        let hash_clock: HashClock = HashClock::default();

        let dummy_vote_cert: VoteCert = VoteCert::new(0, block_hash);
        let block: Block<TransactionVariant> = Block::<TransactionVariant>::new(
            transactions,
            account_pub_key,
            block_hash,
            hash_clock.get_hash_time(),
            dummy_vote_cert,
        );

        let mut block_container: BlockContainer<TransactionVariant> = BlockContainer::new();
        block_container.append_block(block);
    }
}
