//! TODO
//! 1) Add tests for acc crypto

extern crate engine;
extern crate proc;

// use assert_eq::assert_eq;
use rand::rngs::{ThreadRng};

use diem_crypto::{
    traits::{Uniform},
};
use engine::domain::OrderSide;    
use proc::account::{OrderAccount, AccountError, AccountPubKey, AccountPrivKey};
use proc::spot::{SpotController};
use proc::bank::{BankController};


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BrokerAsset {
    BTC,
    USD,
    EUR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_bid() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset: BrokerAsset = BrokerAsset::USD;
        let base_balance: u64 = 1_000_000;
        let quote_balance: u64 = 1_000_000;

        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut spot_controller: SpotController<BrokerAsset> = SpotController::new(base_asset, quote_asset, &mut bank_controller);
        spot_controller.create_account(&account_pub_key, base_balance, quote_balance).unwrap();

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        spot_controller.place_limit_order(&account_pub_key, OrderSide::Bid, bid_size, bid_price).unwrap();

        let account_0: &OrderAccount =  spot_controller.get_account(&account_pub_key).unwrap();

        assert_eq!(account_0.quote_balance, quote_balance - bid_size * bid_price);
        assert_eq!(account_0.quote_escrow, bid_size * bid_price);

        assert_eq!(account_0.base_balance, base_balance);
        assert_eq!(account_0.base_escrow, 0);
    }

    #[test]
    fn place_ask() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance:u64 = 1_000_000;
        let quote_balance:u64 = 1_000_000;

        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut spot_controller: SpotController<BrokerAsset> = SpotController::new(base_asset, quote_asset, &mut bank_controller);
        spot_controller.create_account(&account_pub_key, base_balance, quote_balance).unwrap();

        let bid_size: u64 = 100;
        let bid_price: u64 = 100;
        spot_controller.place_limit_order(&account_pub_key,  OrderSide::Ask, bid_size, bid_price).unwrap();

        let account_0: &OrderAccount =  spot_controller.get_account(&account_pub_key).unwrap();

        assert_eq!(account_0.quote_balance, quote_balance);
        assert_eq!(account_0.quote_escrow, 0);

        assert_eq!(account_0.base_balance, base_balance - bid_size);
        assert_eq!(account_0.base_escrow, bid_size);
    }
    
    #[test]
    fn fail_on_invalid_account_lookup() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;

        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut spot_controller: SpotController<BrokerAsset> = SpotController::new(base_asset, quote_asset, &mut bank_controller);
        let result: AccountError = spot_controller.get_account(&account_pub_key).unwrap_err();

        assert!(matches!(result, AccountError::Lookup(_)));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance: u64 = 1_000_000;
        let quote_balance: u64 = 1_000_000;

        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();

        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut spot_controller: SpotController<BrokerAsset> = SpotController::new(base_asset, quote_asset, &mut bank_controller);
        spot_controller.create_account(&account_pub_key, base_balance, quote_balance).unwrap();

        let result: AccountError = spot_controller.create_account(&account_pub_key, base_balance, quote_balance).unwrap_err();
        assert!(matches!(result, AccountError::Creation(_)));
    }

    #[test]
    fn multi_bid() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance_0: u64 = 1_000_000;
        let quote_balance_0: u64 = 1_000_000;
        let base_balance_1: u64 = 1_000_000;
        let quote_balance_1: u64 = 1_000_000;

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
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, 0, 500_000_000);
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, 1, 500_000_000);

        let mut spot_controller: SpotController<BrokerAsset> = SpotController::new(base_asset, quote_asset, &mut bank_controller);
        spot_controller.create_account(&account_pub_key_0, base_balance_0, quote_balance_0).unwrap();
        spot_controller.create_account(&account_pub_key_1, base_balance_1, quote_balance_1).unwrap();

        let bid_size_0: u64 = 100;
        let bid_price_0: u64 = 100;
        spot_controller.place_limit_order(&account_pub_key_0,  OrderSide::Bid, bid_size_0, bid_price_0).unwrap();

        let bid_size_1: u64 = 10;
        let bid_price_1: u64 = 10;
        spot_controller.place_limit_order(&account_pub_key_1,  OrderSide::Bid, bid_size_1, bid_price_1).unwrap();

        let account_0: &OrderAccount =  spot_controller.get_account(&account_pub_key_0).unwrap();

        assert_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_eq!(account_0.quote_escrow, bid_size_0 * bid_price_0);

        let account_1: &OrderAccount =  spot_controller.get_account(&account_pub_key_1).unwrap();

        assert_eq!(account_1.quote_balance, quote_balance_1 - bid_size_1 * bid_price_1);
        assert_eq!(account_1.quote_escrow, bid_size_1 * bid_price_1);

    }


    #[test]
    fn multi_bid_and_ask() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance_0: u64 = 1_000_000;
        let quote_balance_0: u64 = 1_000_000;
        let base_balance_1: u64 = 1_000_000;
        let quote_balance_1: u64 = 1_000_000;

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
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, 0, 500_000_000);
        bank_controller.transfer(&account_pub_key_0, &account_pub_key_1, 1, 500_000_000);

        let mut spot_controller: SpotController<BrokerAsset> = SpotController::new(base_asset, quote_asset, &mut bank_controller);
        // TODO - check & handle account creation error
        spot_controller.create_account(&account_pub_key_0, base_balance_0, quote_balance_0).unwrap();
        spot_controller.create_account(&account_pub_key_1, base_balance_1, quote_balance_1).unwrap();

        // Place bid for account 0
        let bid_size_0: u64 = 95;
        let bid_price_0: u64 = 200;
        spot_controller.place_limit_order(&account_pub_key_0,  OrderSide::Bid, bid_size_0, bid_price_0).unwrap();

        
        let account_0: &OrderAccount =  spot_controller.get_account(&account_pub_key_0).unwrap();
        assert_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_eq!(account_0.quote_escrow, bid_size_0 * bid_price_0);


        // Place bid for account 1 behind account 0
        let bid_size_1: u64 = bid_size_0 - 2;
        let bid_price_1: u64 = bid_price_0 - 2;
        spot_controller.place_limit_order(&account_pub_key_1,  OrderSide::Bid, bid_size_1, bid_price_1).unwrap();

        let account_1: &OrderAccount =  spot_controller.get_account(&account_pub_key_1).unwrap();
        assert_eq!(account_1.quote_balance, quote_balance_1 - bid_size_1 * bid_price_1);
        assert_eq!(account_1.quote_escrow, bid_size_1 * bid_price_1);


        // Place ask for account 1 at price that crosses spread entirely
        let ask_size_0: u64 = bid_size_0;
        let ask_price_0: u64 = bid_price_0 - 1;
        spot_controller.place_limit_order(&account_pub_key_1,  OrderSide::Ask, ask_size_0, ask_price_0).unwrap();

        let account_0: &OrderAccount =  spot_controller.get_account(&account_pub_key_0).unwrap();
        let account_1: &OrderAccount =  spot_controller.get_account(&account_pub_key_1).unwrap();

        // check account 0
        // paid bid_size_0 * bid_price_0 in quote asset to balance
        // received bid_size_0 in base asset to balance
        assert_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_eq!(account_0.quote_escrow, 0);

        assert_eq!(account_0.base_balance, base_balance_0 + bid_size_0);
        assert_eq!(account_0.base_escrow, 0);

        // check account 1
        // received bid_size_0 * bid_price_0 in quote asset to balance
        // sent bid_size_1 * bid_price_1 in quote asset to escrow
        // paid bid_size_0 in base asset from balance
        assert_eq!(account_1.quote_balance, quote_balance_1 - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0);
        assert_eq!(account_1.quote_escrow, bid_size_1  * bid_price_1);

        assert_eq!(account_1.base_balance, base_balance_1 - bid_size_0);
        assert_eq!(account_1.base_escrow, 0);

        
        // Place second ask for account 1 at price that crosses spread entirely
        let ask_size_1: u64 = bid_size_1;
        let ask_price_1: u64 = bid_price_1 - 1;
        spot_controller.place_limit_order(&account_pub_key_1, OrderSide::Ask, ask_size_1, ask_price_1).unwrap();

        let account_0: &OrderAccount = spot_controller.get_account(&account_pub_key_0).unwrap();
        let account_1: &OrderAccount = spot_controller.get_account(&account_pub_key_1).unwrap();

        // check account 0
        // state should remain unchanged from prior
        assert_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_eq!(account_0.quote_escrow, 0);

        assert_eq!(account_0.base_balance, base_balance_0 + bid_size_0);
        assert_eq!(account_0.base_escrow, 0);


        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_eq!(account_1.quote_balance, quote_balance_1 + bid_size_0 * bid_price_0);
        assert_eq!(account_1.quote_escrow, 0);

        assert_eq!(account_1.base_balance, base_balance_1 - bid_size_0);
        assert_eq!(account_1.base_escrow, 0);

    }
    // TODO - add more tests...
}