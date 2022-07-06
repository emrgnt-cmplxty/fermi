
extern crate engine;
extern crate proc;

use engine::orderbook::{Orderbook};
use engine::domain::OrderSide;    
use proc::account::{Account, AccountError, AccountController};
use assert_approx_eq::assert_approx_eq;

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
        let base_balance: f64 = 1_000_000.0;
        let quote_balance: f64 = 1_000_000.0;
        let account_pub_key: u64 = 0;

        // let mut orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);
        let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

        market_controller.create_account(account_pub_key, base_balance, quote_balance).unwrap();

        let bid_size: f64 = 100.0;
        let bid_price: f64 = 100.0;
        market_controller.place_limit_order(account_pub_key, OrderSide::Bid, bid_size, bid_price).unwrap();

        let account_0: &Account =  market_controller.get_account(account_pub_key).unwrap();

        assert_approx_eq!(account_0.quote_balance, quote_balance - bid_size * bid_price);
        assert_approx_eq!(account_0.quote_escrow, bid_size * bid_price);

        assert_approx_eq!(account_0.base_balance, base_balance);
        assert_approx_eq!(account_0.base_escrow, 0.0);
    }

    #[test]
    fn place_ask() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance:f64 = 1_000_000.0;
        let quote_balance:f64 = 1_000_000.0;
        let account_pub_key:u64 = 0;

        let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

        market_controller.create_account(account_pub_key, base_balance, quote_balance).unwrap();

        let bid_size: f64 = 100.0;
        let bid_price: f64 = 100.0;
        market_controller.place_limit_order(account_pub_key,  OrderSide::Ask, bid_size, bid_price).unwrap();

        let account_0: &Account =  market_controller.get_account(account_pub_key).unwrap();

        assert_approx_eq!(account_0.quote_balance, quote_balance);
        assert_approx_eq!(account_0.quote_escrow, 0.0);

        assert_approx_eq!(account_0.base_balance, base_balance - bid_size);
        assert_approx_eq!(account_0.base_escrow, bid_size);
    }
    
    #[test]
    fn fail_on_invalid_account_lookup() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;

        let account_pub_key:u64 = 0;
        let market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

        let result: AccountError = market_controller.get_account(account_pub_key).unwrap_err();
        assert!(matches!(result, AccountError::Lookup(_)));
    }

    #[test]
    fn fail_on_account_double_creation() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance: f64 = 1_000_000.0;
        let quote_balance: f64 = 1_000_000.0;

        let account_pub_key:u64 = 0;
        let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

        market_controller.create_account(account_pub_key, base_balance, quote_balance).unwrap();
        let result: AccountError = market_controller.create_account(account_pub_key, base_balance, quote_balance).unwrap_err();
        assert!(matches!(result, AccountError::Creation(_)));
    }

    #[test]
    fn multi_bid() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance_0: f64 = 1_000_000.0;
        let quote_balance_0: f64 = 1_000_000.0;
        let account_id_0: u64 = 0;
        let base_balance_1: f64 = 1_000_000.0;
        let quote_balance_1: f64 = 1_000_000.0;
        let account_id_1: u64 = 1;

        let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

        market_controller.create_account(account_id_0, base_balance_0, quote_balance_0).unwrap();
        market_controller.create_account(account_id_1, base_balance_1, quote_balance_1).unwrap();

        let bid_size_0: f64 = 100.0;
        let bid_price_0: f64 = 100.0;
        market_controller.place_limit_order(account_id_0,  OrderSide::Bid, bid_size_0, bid_price_0).unwrap();


        let bid_size_1: f64 = 10.0;
        let bid_price_1: f64 = 10.0;
        market_controller.place_limit_order(account_id_1,  OrderSide::Bid, bid_size_1, bid_price_1).unwrap();

        let account_0: &Account =  market_controller.get_account(account_id_0).unwrap();

        assert_approx_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_approx_eq!(account_0.quote_escrow, bid_size_0 * bid_price_0);

        let account_1: &Account =  market_controller.get_account(account_id_1).unwrap();

        assert_approx_eq!(account_1.quote_balance, quote_balance_1 - bid_size_1 * bid_price_1);
        assert_approx_eq!(account_1.quote_escrow, bid_size_1 * bid_price_1);

    }


    #[test]
    fn multi_bid_and_ask() {
        let base_asset:BrokerAsset = BrokerAsset::BTC;
        let quote_asset:BrokerAsset = BrokerAsset::USD;
        let base_balance_0: f64 = 1_000_000.0;
        let quote_balance_0: f64 = 1_000_000.0;
        let account_id_0: u64 = 0;
        let base_balance_1: f64 = 1_000_000.0;
        let quote_balance_1: f64 = 1_000_000.0;
        let account_id_1: u64 = 1;

        let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

        // TODO - check & handle account creation error
        market_controller.create_account(account_id_0, base_balance_0, quote_balance_0).unwrap();
        market_controller.create_account(account_id_1, base_balance_1, quote_balance_1).unwrap();

        // Place bid for account 0
        let bid_size_0: f64 = 95.3;
        let bid_price_0: f64 = 200.1;
        market_controller.place_limit_order(account_id_0,  OrderSide::Bid, bid_size_0, bid_price_0).unwrap();

        
        let account_0: &Account =  market_controller.get_account(account_id_0).unwrap();
        assert_approx_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_approx_eq!(account_0.quote_escrow, bid_size_0 * bid_price_0);


        // Place bid for account 1 behind account 0
        let bid_size_1: f64 = bid_size_0 - 2.;
        let bid_price_1: f64 = bid_price_0 - 2.;
        market_controller.place_limit_order(account_id_1,  OrderSide::Bid, bid_size_1, bid_price_1).unwrap();

        let account_1: &Account =  market_controller.get_account(account_id_1).unwrap();
        assert_approx_eq!(account_1.quote_balance, quote_balance_1 - bid_size_1 * bid_price_1);
        assert_approx_eq!(account_1.quote_escrow, bid_size_1 * bid_price_1);


        // Place ask for account 1 at price that crosses spread entirely
        let ask_size_0: f64 = bid_size_0;
        let ask_price_0: f64 = bid_price_0 - 1.;
        market_controller.place_limit_order(account_id_1,  OrderSide::Ask, ask_size_0, ask_price_0).unwrap();

        let account_0: &Account =  market_controller.get_account(account_id_0).unwrap();
        let account_1: &Account =  market_controller.get_account(account_id_1).unwrap();

        // check account 0
        // paid bid_size_0 * bid_price_0 in quote asset to balance
        // received bid_size_0 in base asset to balance
        assert_approx_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_approx_eq!(account_0.quote_escrow, 0.);

        assert_approx_eq!(account_0.base_balance, base_balance_0 + bid_size_0);
        assert_approx_eq!(account_0.base_escrow, 0.);

        // check account 1
        // received bid_size_0 * bid_price_0 in quote asset to balance
        // sent bid_size_1 * bid_price_1 in quote asset to escrow
        // paid bid_size_0 in base asset from balance
        assert_approx_eq!(account_1.quote_balance, quote_balance_1 - bid_size_1 * bid_price_1 + bid_size_0 * bid_price_0);
        assert_approx_eq!(account_1.quote_escrow, bid_size_1  * bid_price_1);

        assert_approx_eq!(account_1.base_balance, base_balance_1 - bid_size_0);
        assert_approx_eq!(account_1.base_escrow, 0.);

        
        // Place second ask for account 1 at price that crosses spread entirely
        let ask_size_1: f64 = bid_size_1;
        let ask_price_1: f64 = bid_price_1 - 1.;
        market_controller.place_limit_order(account_id_1, OrderSide::Ask, ask_size_1, ask_price_1).unwrap();

        let account_0: &Account = market_controller.get_account(account_id_0).unwrap();
        let account_1: &Account = market_controller.get_account(account_id_1).unwrap();

        // check account 0
        // state should remain unchanged from prior
        assert_approx_eq!(account_0.quote_balance, quote_balance_0 - bid_size_0 * bid_price_0);
        assert_approx_eq!(account_0.quote_escrow, 0.);

        assert_approx_eq!(account_0.base_balance, base_balance_0 + bid_size_0);
        assert_approx_eq!(account_0.base_escrow, 0.);


        // check account 1
        // additional trade should act to move bid_size_1 * bid_price_1 in quote from escrow to balance
        assert_approx_eq!(account_1.quote_balance, quote_balance_1 + bid_size_0 * bid_price_0);
        assert_approx_eq!(account_1.quote_escrow, 0.);

        assert_approx_eq!(account_1.base_balance, base_balance_1 - bid_size_0);
        assert_approx_eq!(account_1.base_escrow, 0.);

    }
    // TODO - add more tests...
}