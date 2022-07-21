//! TODO
//! 1.) ADD TESTS FOR CRYPTOGRAPHIC VERIFICATINO
//! 2.) MOVE TOWARDS PRE-DETRMINED KEYS INSTEAD OF RNG
//!

#[cfg(test)]
mod tests {
    extern crate proc;
    extern crate types;

    use gdex_crypto::traits::Uniform;
    use proc::{
        bank::{BankController, CREATED_ASSET_BALANCE},
        spot::OrderbookInterface,
    };
    use rand::rngs::ThreadRng;
    use types::{
        account::{AccountPrivKey, AccountPubKey},
        asset::AssetId,
        error::GDEXError,
        orderbook::OrderSide,
    };

    const TRANSFER_AMOUNT: u64 = 1_000_000;

    #[test]
    fn place_bid() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();
        let base_asset_id: AssetId = bank_controller.create_asset(&account_pub_key).unwrap();
        let quote_asset_id: AssetId = bank_controller.create_asset(&account_pub_key).unwrap();

        let mut orderbook_interface: OrderbookInterface = OrderbookInterface::new(base_asset_id, quote_asset_id);

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key,
                OrderSide::Bid,
                bid_size,
                bid_price,
            )
            .unwrap();

        assert_eq!(
            bank_controller.get_balance(&account_pub_key, base_asset_id).unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key, quote_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - bid_size * bid_price
        );
    }

    #[test]
    fn place_ask() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();
        let base_asset_id: AssetId = bank_controller.create_asset(&account_pub_key).unwrap();
        let quote_asset_id: AssetId = bank_controller.create_asset(&account_pub_key).unwrap();

        let mut orderbook_interface: OrderbookInterface = OrderbookInterface::new(base_asset_id, quote_asset_id);

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key,
                OrderSide::Ask,
                bid_size,
                bid_price,
            )
            .unwrap();

        assert_eq!(
            bank_controller.get_balance(&account_pub_key, quote_asset_id).unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key, base_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - bid_size
        );
    }

    #[test]
    fn fail_on_invalid_account_lookup() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let base_asset_id: AssetId = 0;
        let quote_asset_id: AssetId = 1;
        let orderbook_interface: OrderbookInterface = OrderbookInterface::new(base_asset_id, quote_asset_id);
        let result: GDEXError = orderbook_interface.get_account(&account_pub_key).unwrap_err();

        assert!(matches!(result, GDEXError::Lookup(_)));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let base_asset_id: AssetId = 0;
        let quote_asset_id: AssetId = 1;
        let mut orderbook_interface: OrderbookInterface = OrderbookInterface::new(base_asset_id, quote_asset_id);
        orderbook_interface.create_account(&account_pub_key).unwrap();
        let result: GDEXError = orderbook_interface.create_account(&account_pub_key).unwrap_err();
        assert!(matches!(result, GDEXError::Creation(_)));
    }

    #[test]
    fn multi_bid() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_0: AccountPubKey = (&private_key).into();

        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_1: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();

        let base_asset_id: AssetId = bank_controller.create_asset(&account_pub_key_0).unwrap();
        let quote_asset_id: AssetId = bank_controller.create_asset(&account_pub_key_0).unwrap();
        bank_controller
            .transfer(&account_pub_key_0, &account_pub_key_1, base_asset_id, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(&account_pub_key_0, &account_pub_key_1, quote_asset_id, TRANSFER_AMOUNT)
            .unwrap();

        let mut orderbook_interface: OrderbookInterface = OrderbookInterface::new(base_asset_id, quote_asset_id);

        let bid_size_0: u64 = 100;
        let bid_price_0: u64 = 100;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key_0,
                OrderSide::Bid,
                bid_size_0,
                bid_price_0,
            )
            .unwrap();

        let bid_size_1: u64 = 110;
        let bid_price_1: u64 = 110;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key_1,
                OrderSide::Bid,
                bid_size_1,
                bid_price_1,
            )
            .unwrap();

        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, quote_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, base_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, quote_asset_id).unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, base_asset_id).unwrap(),
            TRANSFER_AMOUNT
        );
    }

    #[test]
    fn multi_bid_and_ask() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_0: AccountPubKey = (&private_key).into();

        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_1: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();

        let base_asset_id: AssetId = bank_controller.create_asset(&account_pub_key_0).unwrap();
        let quote_asset_id: AssetId = bank_controller.create_asset(&account_pub_key_0).unwrap();
        bank_controller
            .transfer(&account_pub_key_0, &account_pub_key_1, base_asset_id, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(&account_pub_key_0, &account_pub_key_1, quote_asset_id, TRANSFER_AMOUNT)
            .unwrap();

        let mut orderbook_interface: OrderbookInterface = OrderbookInterface::new(base_asset_id, quote_asset_id);

        let bid_size_0: u64 = 95;
        let bid_price_0: u64 = 200;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key_0,
                OrderSide::Bid,
                bid_size_0,
                bid_price_0,
            )
            .unwrap();

        let bid_size_1: u64 = bid_size_0;
        let bid_price_1: u64 = bid_price_0 - 2;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key_1,
                OrderSide::Bid,
                bid_size_1,
                bid_price_1,
            )
            .unwrap();

        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, quote_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, base_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, quote_asset_id).unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, base_asset_id).unwrap(),
            TRANSFER_AMOUNT
        );

        // Place ask for account 1 at price that crosses spread entirely
        let ask_size_0: u64 = bid_size_0;
        let ask_price_0: u64 = bid_price_0 - 1;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key_1,
                OrderSide::Ask,
                ask_size_0,
                ask_price_0,
            )
            .unwrap();

        // check account 0
        // received initial asset creation balance
        // paid bid_size_0 * bid_price_0 in quote asset to orderbook
        // received bid_size_0 in base asset from settled trade
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, quote_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, base_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // received initial transfer amount
        // received bid_size_0 * bid_price_0 in quote asset to balance
        // sent bid_size_1 * bid_price_1 in quote asset to escrow
        // paid bid_size_0 in base asset from balance
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, quote_asset_id).unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, base_asset_id).unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );

        // Place final order for account 1 at price that crosses spread entirely and closes it's own position
        let ask_size_1: u64 = bid_size_1;
        let ask_price_1: u64 = bid_price_1 - 1;
        orderbook_interface
            .place_limit_order(
                &mut bank_controller,
                &account_pub_key_1,
                OrderSide::Ask,
                ask_size_1,
                ask_price_1,
            )
            .unwrap();

        // check account 0
        // state should remain unchanged from prior
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, quote_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_0, base_asset_id).unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, quote_asset_id).unwrap(),
            TRANSFER_AMOUNT + bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller.get_balance(&account_pub_key_1, base_asset_id).unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );
    }
}
