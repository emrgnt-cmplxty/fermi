#[cfg(test)]
pub mod futures_tests {
    // crate
    use crate::futures::{proto::*, types::*};
    use crate::router::ControllerRouter;
    use crate::ControllerTestBed;
    // gdex
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, AccountKeyPair, AccountPubKey},
        asset::AssetId,
        crypto::KeypairTraits,
        error::GDEXError,
        order_book::OrderSide,
        transaction::Transaction,
    };
    // mysten
    use narwhal_types::CertificateDigest;
    // external
    use std::convert::TryInto;

    // setup constants
    const BASE_ASSET_ID: AssetId = 0;
    const QUOTE_ASSET_ID: AssetId = 1;
    const TEST_MAX_LEVERAGE: u64 = 25;
    const INITIAL_TIME: u64 = 1_000_000;
    const INITIAL_ASSET_PRICES: &'static [u64] = &[11_000_000];
    const ADMIN_INITIAL_DEPOSIT: AssetId = 100_000_000_000;
    const USER_INITIAL_DEPOSIT: AssetId = 10_000_000_000;
    const NUM_USER_ACCOUNTS: usize = 10;
    const FINAL_ASSET_PRICES: &'static [u64] = &[12_000_000];

    pub struct FuturesControllerTester {
        main_controller: ControllerRouter,
        admin_key: AccountKeyPair,
        user_keys: Vec<AccountKeyPair>,
        base_asset_id: u64,
        quote_asset_id: u64,
    }

    impl FuturesControllerTester {
        pub fn new() -> Self {
            let admin_key = generate_keypair_vec([0; 32]).pop().unwrap();
            let mut user_keys = Vec::new();
            for i in 0..NUM_USER_ACCOUNTS {
                let user_key = generate_keypair_vec([i as u8; 32]).pop().unwrap();
                user_keys.push(user_key);
            }

            Self {
                main_controller: ControllerRouter::default(),
                admin_key,
                user_keys,
                base_asset_id: BASE_ASSET_ID,
                quote_asset_id: QUOTE_ASSET_ID,
            }
        }

        fn initialize_bank_controller(&self) -> Result<(), GDEXError> {
            let mut bank_controller = self.main_controller.bank_controller.lock().unwrap();
            // create two assets for the base and quote of the futures market
            bank_controller.create_asset(self.admin_key.public()).unwrap();
            bank_controller.create_asset(self.admin_key.public()).unwrap();
            for user_key in self.user_keys.iter() {
                bank_controller.transfer(
                    self.admin_key.public(),
                    user_key.public(),
                    BASE_ASSET_ID,
                    USER_INITIAL_DEPOSIT,
                )?;
                bank_controller.transfer(
                    self.admin_key.public(),
                    user_key.public(),
                    QUOTE_ASSET_ID,
                    USER_INITIAL_DEPOSIT,
                )?;
            }
            Ok(())
        }

        fn create_marketplace(&self) -> Result<(), GDEXError> {
            let request = CreateMarketplaceRequest::new(self.quote_asset_id);
            let transaction = Transaction::new(
                &self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        fn create_market(&self) -> Result<(), GDEXError> {
            let request = CreateMarketRequest::new(self.base_asset_id);
            let transaction = Transaction::new(
                &self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        fn update_market_params(&self) -> Result<(), GDEXError> {
            let request = UpdateMarketParamsRequest::new(self.base_asset_id, TEST_MAX_LEVERAGE);
            let transaction = Transaction::new(
                &self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        fn update_time(&self, latest_time: u64) -> Result<(), GDEXError> {
            let request = UpdateTimeRequest::new(latest_time);
            let transaction = Transaction::new(
                &self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        fn update_prices(&self, latest_prices: Vec<u64>) -> Result<(), GDEXError> {
            let request = UpdatePricesRequest::new(latest_prices);
            let transaction = Transaction::new(
                &self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        fn account_deposit(&self, quantity: u64, sender: AccountPubKey) -> Result<(), GDEXError> {
            let request = AccountDepositRequest::new(
                quantity.try_into().map_err(|_| GDEXError::Conversion)?,
                &self.admin_key.public(),
            );
            let transaction = Transaction::new(
                &sender,
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        fn initialize_futures_controller(&self) -> Result<(), GDEXError> {
            self.create_marketplace()?;
            self.create_market()?;
            self.update_market_params()?;
            self.update_time(INITIAL_TIME)?;
            self.update_prices(INITIAL_ASSET_PRICES.to_vec())?;
            self.account_deposit(ADMIN_INITIAL_DEPOSIT, self.admin_key.public().clone())?;
            for user_key in self.user_keys.iter() {
                self.account_deposit(USER_INITIAL_DEPOSIT, user_key.public().clone())?;
            }
            Ok(())
        }

        fn futures_limit_order(
            &self,
            user_index: usize,
            side: u64,
            price: u64,
            quantity: u64,
        ) -> Result<(), GDEXError> {
            let request = FuturesLimitOrderRequest::new(
                self.base_asset_id,
                self.quote_asset_id,
                side,
                price,
                quantity,
                &self.admin_key.public(),
            );

            let transaction = Transaction::new(
                &self.user_keys[user_index].public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.main_controller.handle_consensus_transaction(&transaction)
        }

        // UTIILTY FUNCTIONS
        fn get_user_total_req_collateral(&self, user_index: usize) -> Result<u64, GDEXError> {
            self.main_controller
                .futures_controller
                .lock()
                .unwrap()
                .account_total_req_collateral(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_user_open_positions(&self, user_index: usize) -> Result<Vec<Position>, GDEXError> {
            self.main_controller
                .futures_controller
                .lock()
                .unwrap()
                .account_open_market_positions(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_user_unrealized_pnl(&self, user_index: usize) -> Result<i64, GDEXError> {
            self.main_controller
                .futures_controller
                .lock()
                .unwrap()
                .account_unrealized_pnl(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_account_available_deposit(&self, user_index: usize) -> Result<i64, GDEXError> {
            self.main_controller
                .futures_controller
                .lock()
                .unwrap()
                .account_available_deposit(self.admin_key.public(), self.user_keys[user_index].public())
        }
    }

    impl ControllerTestBed for FuturesControllerTester {
        fn get_main_controller(&self) -> &ControllerRouter {
            &self.main_controller
        }
        fn initialize(&self) {
            self.generic_initialize();
            self.initialize_bank_controller().unwrap();
            self.initialize_futures_controller().unwrap();
        }
    }

    #[test]
    fn initialize() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
    }

    #[test]
    fn place_order() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (user_index, user_side, user_price, user_quantity) = (0, OrderSide::Bid as u64, 10_000_000, 100);

        futures_tester
            .futures_limit_order(user_index, user_side, user_price, user_quantity)
            .unwrap();

        let req_collateral = futures_tester.get_user_total_req_collateral(user_index).unwrap();
        // assert order used collateral equal to price times quantity // leverage + 1
        assert!(req_collateral == (user_price * user_quantity) / TEST_MAX_LEVERAGE + 1);
    }

    #[test]
    fn cross_spread() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Bid as u64, 10_000_000, 100);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Ask as u64, 10_000_000 - 1, 10);

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        let maker_position = futures_tester
            .get_user_open_positions(maker_index)
            .unwrap()
            .pop()
            .unwrap();
        let taker_position = futures_tester
            .get_user_open_positions(taker_index)
            .unwrap()
            .pop()
            .unwrap();
        // check taker and maker positions match
        assert!(maker_position.quantity == taker_position.quantity);
        assert!(maker_position.average_price == taker_position.average_price);
        // now check taker was completely filled
        assert!(maker_position.quantity == taker_quantity);
        // note, average price should be at maker fill price
        assert!(maker_position.average_price == maker_price);

        // check taker and maker collateral requirements are consistent w/ expected state
        let maker_total_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let taker_total_req_collateral = futures_tester.get_user_total_req_collateral(taker_index).unwrap();
        let maker_order_req_collateral = (maker_quantity - taker_quantity) * maker_price / TEST_MAX_LEVERAGE;
        // maker required collateral should equal taker required collateral + remaining order collateral
        assert!(maker_total_req_collateral - maker_order_req_collateral == taker_total_req_collateral);

        let maker_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let taker_unrealized_pnl = futures_tester.get_user_unrealized_pnl(taker_index).unwrap();
        // check that maker and taker unrealized pnl off-set
        assert!(maker_unrealized_pnl == -1 * taker_unrealized_pnl);
        // check that pnl markdown equals difference from fill price to current price
        assert!(maker_unrealized_pnl as u64 == (INITIAL_ASSET_PRICES[0] - 10_000_000) * taker_quantity);
    }

    #[test]
    fn exceed_collateral() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (user_index, user_side, user_price, user_quantity) =
            (0, OrderSide::Bid as u64, 1_000_000_000, 1_000_000_000);

        let result = futures_tester.futures_limit_order(user_index, user_side, user_price, user_quantity);

        assert!(result.unwrap_err() == GDEXError::InsufficientCollateral);
    }

    #[test]
    fn multi_order() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (user_index, user_side, user_price_0, user_quantity_0) = (0, OrderSide::Bid as u64, 1_000_000, 1_000);
        futures_tester
            .futures_limit_order(user_index, user_side, user_price_0, user_quantity_0)
            .unwrap();

        let (user_price_1, user_quantity_1) = (2_000_000, 1_000);
        futures_tester
            .futures_limit_order(user_index, user_side, user_price_1, user_quantity_1)
            .unwrap();

        // check that collateral requirements of orders sums over price*quantity
        let user_req_collateral = futures_tester.get_user_total_req_collateral(user_index).unwrap();
        assert!(
            user_req_collateral
                == (user_price_0 * user_quantity_0 + user_price_1 * user_quantity_1) / TEST_MAX_LEVERAGE + 1
        );

        // a bid on opposite side which does not cross spread will not modify req collateral
        // unless that bid causes max collateral req on associated side to exceed the other side
        let (user_side, user_price_2, user_quantity_2) = (OrderSide::Ask as u64, 3_000_000, 1);
        futures_tester
            .futures_limit_order(user_index, user_side, user_price_2, user_quantity_2)
            .unwrap();
        let user_req_collateral_post_ask = futures_tester.get_user_total_req_collateral(user_index).unwrap();
        assert!(user_req_collateral == user_req_collateral_post_ask);

        // an ask that crosses will modify the required collateral by removing open orders
        let (user_side, user_price_3, user_quantity_3) = (OrderSide::Ask as u64, 999_000, 25);
        futures_tester
            .futures_limit_order(user_index, user_side, user_price_3, user_quantity_3)
            .unwrap();

        // TODO - the assert below works, but residual rounding from price calculation has been added back in
        // can we easily calculate this dynamically?
        // e.g. in this example the residual is floor(1975 * .886...) = 70, where
        // the residual .886 comes from the rounding on price = quantity_1 * price_1 + quantity_2 * price_2 / quantity_1 + quantity_2
        let user_req_collateral_post_cross = futures_tester.get_user_total_req_collateral(user_index).unwrap();

        assert!(
            user_req_collateral
                == user_req_collateral_post_cross + (user_price_1 * user_quantity_3) / TEST_MAX_LEVERAGE + 70
        );

        // ensure that we don't have a remainder transaction
        let user_position = futures_tester.get_user_open_positions(user_index).unwrap().pop();
        assert!(user_position.is_none())
    }

    #[test]
    fn cross_spread_and_tick_prices() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Bid as u64, 10_000_000, 100);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Ask as u64, 10_000_000 - 1, 10);

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        // tick to final prices
        futures_tester.update_prices(FINAL_ASSET_PRICES.to_vec()).unwrap();

        let maker_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();

        // check that pnl markdown equals difference from fill price to current price
        assert!(maker_unrealized_pnl as u64 == (FINAL_ASSET_PRICES[0] - 10_000_000) * taker_quantity);
    }

    #[test]
    fn cross_spread_and_realize_pnl() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Bid as u64, 10_000_000, 10);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Ask as u64, 10_000_000 - 1, 10);

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Ask as u64, 11_000_000, 10);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Bid as u64, 11_000_000 + 1, 10);

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        let maker_position = futures_tester.get_user_open_positions(maker_index).unwrap().pop();
        let taker_position = futures_tester.get_user_open_positions(taker_index).unwrap().pop();

        assert!(maker_position.is_none());
        assert!(taker_position.is_none());

        let maker_available_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let taker_available_deposit = futures_tester.get_account_available_deposit(taker_index).unwrap();

        assert!(maker_available_deposit as u64 == USER_INITIAL_DEPOSIT + 1_000_000 * 10 - 1);
        assert!(taker_available_deposit as u64 == USER_INITIAL_DEPOSIT - 1_000_000 * 10 - 1);
    }
}
