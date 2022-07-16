//! TODO
//! 1.) Add tests for acc crypto
//! 2.) Move to pre-determined keys to avoid use of RNG
//! 
extern crate proc;
extern crate types;

use rand::rngs::{ThreadRng};

use diem_crypto::{
    traits::{Uniform},
};
use proc::{
    spot::{SpotController},
    bank::{BankController}
};
use types::{
    account::{AccountError, AccountPubKey, AccountPrivKey},
    orderbook::OrderSide
};

#[cfg(test)]
mod tests {
    use super::*;
    const BASE_ASSET_ID: u64 = 0;
    const QUOTE_ASSET_ID: u64 = 1;
    const CREATED_ASSET_BALANCE: u64 = 1_000_000_000_000;
    const TRANSFER_AMOUNT: u64 = 1_000_000;

    #[test]
    fn place_bid() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID, &mut bank_controller);
        spot_controller.create_account(&account_pub_key).unwrap();

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        spot_controller.place_limit_order(&account_pub_key, OrderSide::Bid, bid_size, bid_price).unwrap();

        assert_eq!(bank_controller.get_balance(&account_pub_key, QUOTE_ASSET_ID), CREATED_ASSET_BALANCE - bid_size * bid_price);
        assert_eq!(bank_controller.get_balance(&account_pub_key, BASE_ASSET_ID), CREATED_ASSET_BALANCE);
    }

    #[test]
    fn place_ask() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID, &mut bank_controller);
        spot_controller.create_account(&account_pub_key).unwrap();

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        spot_controller.place_limit_order(&account_pub_key, OrderSide::Ask, bid_size, bid_price).unwrap();

        assert_eq!(bank_controller.get_balance(&account_pub_key, QUOTE_ASSET_ID), CREATED_ASSET_BALANCE);
        assert_eq!(bank_controller.get_balance(&account_pub_key, BASE_ASSET_ID), CREATED_ASSET_BALANCE - bid_size);
    }
    
    #[test]
    fn fail_on_invalid_account_lookup() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();

        let spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID, &mut bank_controller);
        let result: AccountError = spot_controller.get_account(&account_pub_key).unwrap_err();

        assert!(matches!(result, AccountError::Lookup(_)));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();

        let mut spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID, &mut bank_controller);
        spot_controller.create_account(&account_pub_key).unwrap();
        let result: AccountError = spot_controller.create_account(&account_pub_key).unwrap_err();
        assert!(matches!(result, AccountError::Creation(_)));
    }

    #[test]
    fn multi_bid() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_0: AccountPubKey = (&private_key).into();

        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_1: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();

        bank_controller.create_account(&account_pub_key_0).unwrap();
        bank_controller.create_account(&account_pub_key_1).unwrap();
        bank_controller.create_asset(&account_pub_key_0);
        bank_controller.create_asset(&account_pub_key_0);
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, BASE_ASSET_ID, TRANSFER_AMOUNT);
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, QUOTE_ASSET_ID, TRANSFER_AMOUNT);

        let mut spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID, &mut bank_controller);

        let bid_size_0: u64 = 100;
        let bid_price_0: u64 = 100;
        spot_controller.place_limit_order(&account_pub_key_0, OrderSide::Bid, bid_size_0, bid_price_0).unwrap();

        let bid_size_1: u64 = 110;
        let bid_price_1: u64 = 110;
        spot_controller.place_limit_order(&account_pub_key_1, OrderSide::Bid, bid_size_1, bid_price_1).unwrap();

        assert_eq!(bank_controller.get_balance(&account_pub_key_0, QUOTE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0);
        assert_eq!(bank_controller.get_balance(&account_pub_key_0, BASE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT);

        assert_eq!(bank_controller.get_balance(&account_pub_key_1, QUOTE_ASSET_ID), TRANSFER_AMOUNT - bid_size_1 * bid_price_1);
        assert_eq!(bank_controller.get_balance(&account_pub_key_1, BASE_ASSET_ID), TRANSFER_AMOUNT);
    }

    #[test]
    fn multi_bid_and_ask() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_0: AccountPubKey = (&private_key).into();

        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key_1: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();

        bank_controller.create_account(&account_pub_key_0).unwrap();
        bank_controller.create_account(&account_pub_key_1).unwrap();
        bank_controller.create_asset(&account_pub_key_0);
        bank_controller.create_asset(&account_pub_key_0);
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, BASE_ASSET_ID, TRANSFER_AMOUNT);
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, QUOTE_ASSET_ID, TRANSFER_AMOUNT);

        let mut spot_controller: SpotController = SpotController::new(BASE_ASSET_ID, QUOTE_ASSET_ID, &mut bank_controller);

        let bid_size_0: u64 = 95;
        let bid_price_0: u64 = 200;
        spot_controller.place_limit_order(&account_pub_key_0, OrderSide::Bid, bid_size_0, bid_price_0).unwrap();

        let bid_size_1: u64 = bid_size_0;
        let bid_price_1: u64 = bid_price_0 - 2;
        spot_controller.place_limit_order(&account_pub_key_1, OrderSide::Bid, bid_size_1, bid_price_1).unwrap();

        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_0, QUOTE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0);
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_0, BASE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT);

        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_1, QUOTE_ASSET_ID), TRANSFER_AMOUNT - bid_size_1 * bid_price_1);
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_1, BASE_ASSET_ID), TRANSFER_AMOUNT);

        // Place ask for account 1 at price that crosses spread entirely
        let ask_size_0: u64 = bid_size_0;
        let ask_price_0: u64 = bid_price_0 - 1;
        spot_controller.place_limit_order(&account_pub_key_1,  OrderSide::Ask, ask_size_0, ask_price_0).unwrap();

        // check account 0
        // received initial asset creation balance
        // paid bid_size_0 * bid_price_0 in quote asset to orderbook
        // received bid_size_0 in base asset from settled trade
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_0, QUOTE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0);
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_0, BASE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0);

        // check account 1
        // received initial transfer amount
        // received bid_size_0 * bid_price_0 in quote asset to balance
        // sent bid_size_1 * bid_price_1 in quote asset to escrow
        // paid bid_size_0 in base asset from balance
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_1, QUOTE_ASSET_ID), TRANSFER_AMOUNT - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0);
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_1, BASE_ASSET_ID), TRANSFER_AMOUNT - bid_size_0);


        // Place final order for account 1 at price that crosses spread entirely and closes it's own position
        let ask_size_1: u64 = bid_size_1;
        let ask_price_1: u64 = bid_price_1 - 1;
        spot_controller.place_limit_order(&account_pub_key_1, OrderSide::Ask, ask_size_1, ask_price_1).unwrap();

        // check account 0
        // state should remain unchanged from prior
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_0, QUOTE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT - bid_size_0 * bid_price_0);
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_0, BASE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT + bid_size_0);

        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_1, QUOTE_ASSET_ID), TRANSFER_AMOUNT + bid_size_0 * bid_price_0);
        assert_eq!(spot_controller.get_bank_controller().get_balance(&account_pub_key_1, BASE_ASSET_ID), TRANSFER_AMOUNT - bid_size_0);
    }
}