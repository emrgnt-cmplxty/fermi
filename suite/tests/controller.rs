//! TODO
//! 1.) ADD TESTS FOR CRYPTOGRAPHIC VERIFICATINO
//! 2.) MOVE TOWARDS PRE-DETRMINED KEYS INSTEAD OF RNG
//!
#[cfg(test)]
pub mod process_tests {
    use gdex_controller::{
        bank::{BankController, CREATED_ASSET_BALANCE},
        spot::{OrderbookInterface, SPOT_CONTROLLER_ACCOUNT_PUBKEY}
    };
    use gdex_types::{
        account::AccountPubKey,
        crypto::ToFromBytes,
        account::account_test_functions::generate_keypair_vec, asset::AssetId, crypto::KeypairTraits, error::GDEXError,
        order_book::OrderSide,
    };
    use std::sync::{Arc, Mutex};

    const BASE_ASSET_ID: AssetId = 0;
    const QUOTE_ASSET_ID: AssetId = 1;
    const TRANSFER_AMOUNT: u64 = 1_000_000;

    #[test]
    fn place_bid() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();
        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref
            .lock()
            .unwrap()
            .create_account(&controller_account);

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, controller_account, Arc::clone(&bank_controller_ref));

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        orderbook_interface
            .place_limit_order(account.public(), OrderSide::Bid, bid_size, bid_price)
            .unwrap();

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size * bid_price
        );
    }

    #[test]
    fn place_ask() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();
        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref
            .lock()
            .unwrap()
            .create_account(&controller_account);

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, controller_account, Arc::clone(&bank_controller_ref));

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        orderbook_interface
            .place_limit_order(account.public(), OrderSide::Ask, bid_size, bid_price)
            .unwrap();

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - bid_size
        );
    }

    #[test]
    fn fail_on_invalid_account_lookup() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();
        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref
            .lock()
            .unwrap()
            .create_account(&controller_account);

        let orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, controller_account, Arc::clone(&bank_controller_ref));

        let result: GDEXError = orderbook_interface.get_account(account.public()).unwrap_err();

        assert!(matches!(result, GDEXError::AccountLookup));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let account = generate_keypair_vec([0; 32]).pop().unwrap();
        let mut bank_controller = BankController::default();
        bank_controller.create_asset(account.public()).unwrap();
        bank_controller.create_asset(account.public()).unwrap();
        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref
            .lock()
            .unwrap()
            .create_account(&controller_account);

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, controller_account, Arc::clone(&bank_controller_ref));

        orderbook_interface.create_account(account.public()).unwrap();
        let result: GDEXError = orderbook_interface.create_account(account.public()).unwrap_err();
        assert!(matches!(result, GDEXError::AccountCreation));
    }

    #[test]
    fn multi_bid() {
        let account_0 = generate_keypair_vec([0; 32]).pop().unwrap();
        let account_1 = generate_keypair_vec([1; 32]).pop().unwrap();
        let mut bank_controller = BankController::default();

        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), BASE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), QUOTE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();

        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));
        
        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref
            .lock()
            .unwrap()
            .create_account(&controller_account);

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, controller_account, Arc::clone(&bank_controller_ref));

        let bid_size_0: u64 = 100;
        let bid_price_0: u64 = 100;
        orderbook_interface
            .place_limit_order(account_0.public(), OrderSide::Bid, bid_size_0, bid_price_0)
            .unwrap();

        let bid_size_1: u64 = 110;
        let bid_price_1: u64 = 110;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Bid, bid_size_1, bid_price_1)
            .unwrap();

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT
        );
    }

    #[test]
    fn multi_bid_and_ask() {
        let account_0 = generate_keypair_vec([0; 32]).pop().unwrap();
        let account_1 = generate_keypair_vec([1; 32]).pop().unwrap();
        let mut bank_controller = BankController::default();

        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller.create_asset(account_0.public()).unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), BASE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();
        bank_controller
            .transfer(account_0.public(), account_1.public(), QUOTE_ASSET_ID, TRANSFER_AMOUNT)
            .unwrap();

        let bank_controller_ref = Arc::new(Mutex::new(bank_controller));

        let controller_account = AccountPubKey::from_bytes(SPOT_CONTROLLER_ACCOUNT_PUBKEY).unwrap();
        let _create_account_result = bank_controller_ref
            .lock()
            .unwrap()
            .create_account(&controller_account);

        let mut orderbook_interface =
            OrderbookInterface::new(BASE_ASSET_ID, QUOTE_ASSET_ID, controller_account, Arc::clone(&bank_controller_ref));

        let bid_size_0: u64 = 95;
        let bid_price_0: u64 = 200;
        orderbook_interface
            .place_limit_order(account_0.public(), OrderSide::Bid, bid_size_0, bid_price_0)
            .unwrap();

        let bid_size_1: u64 = bid_size_0;
        let bid_price_1: u64 = bid_price_0 - 2;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Bid, bid_size_1, bid_price_1)
            .unwrap();

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT
        );

        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT
        );

        // Place ask for account 1 at price that crosses spread entirely
        let ask_size_0: u64 = bid_size_0;
        let ask_price_0: u64 = bid_price_0 - 1;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Ask, ask_size_0, ask_price_0)
            .unwrap();

        // check account 0
        // received initial asset creation balance
        // paid bid_size_0 * bid_price_0 in quote asset to orderbook
        // received bid_size_0 in base asset from settled trade
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // received initial transfer amount
        // received bid_size_0 * bid_price_0 in quote asset to balance
        // sent bid_size_1 * bid_price_1 in quote asset to escrow
        // paid bid_size_0 in base asset from balance
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );

        // Place final order for account 1 at price that crosses spread entirely and closes it's own position
        let ask_size_1: u64 = bid_size_1;
        let ask_price_1: u64 = bid_price_1 - 1;
        orderbook_interface
            .place_limit_order(account_1.public(), OrderSide::Ask, ask_size_1, ask_price_1)
            .unwrap();

        // check account 0
        // state should remain unchanged from prior
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), QUOTE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_0.public(), BASE_ASSET_ID)
                .unwrap(),
            CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0
        );

        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), QUOTE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT + bid_size_0 * bid_price_0
        );
        assert_eq!(
            bank_controller_ref
                .lock()
                .unwrap()
                .get_balance(account_1.public(), BASE_ASSET_ID)
                .unwrap(),
            TRANSFER_AMOUNT - bid_size_0
        );
    }
}
