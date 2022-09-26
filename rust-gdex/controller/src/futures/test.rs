#[allow(dead_code)]
#[allow(unused_imports)] // flags necessary w/ feature flag
#[cfg(any(test, feature = "testing"))]
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
        proto::FuturesPosition,
        transaction::{ExecutionResultBody, Transaction},
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
    const INITIAL_ASSET_PRICES: &[u64] = &[11_000_000];
    const ADMIN_INITIAL_DEPOSIT: AssetId = 100_000_000_000;
    const USER_INITIAL_DEPOSIT: AssetId = 10_000_000_000;
    const NUM_USER_ACCOUNTS: usize = 10;
    const FINAL_ASSET_PRICES: &[u64] = &[12_000_000];

    pub struct FuturesControllerTester {
        pub controller_router: ControllerRouter,
        pub admin_key: AccountKeyPair,
        pub user_keys: Vec<AccountKeyPair>,
        pub base_asset_id: u64,
        pub quote_asset_id: u64,
    }

    impl Default for FuturesControllerTester {
        fn default() -> Self {
            Self::new()
        }
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
                controller_router: ControllerRouter::default(),
                admin_key,
                user_keys,
                base_asset_id: BASE_ASSET_ID,
                quote_asset_id: QUOTE_ASSET_ID,
            }
        }

        fn initialize_bank_controller(&self) -> Result<(), GDEXError> {
            let mut bank_controller = self.controller_router.bank_controller.lock().unwrap();
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

        fn create_marketplace(&self) -> Result<ExecutionResultBody, GDEXError> {
            let request = CreateMarketplaceRequest::new(self.quote_asset_id);
            let transaction = Transaction::new(
                self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        fn create_market(&self) -> Result<ExecutionResultBody, GDEXError> {
            let request = CreateMarketRequest::new(self.base_asset_id);
            let transaction = Transaction::new(
                self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        fn update_market_params(&self) -> Result<ExecutionResultBody, GDEXError> {
            let request = UpdateMarketParamsRequest::new(self.base_asset_id, TEST_MAX_LEVERAGE);
            let transaction = Transaction::new(
                self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        fn update_time(&self, latest_time: u64) -> Result<ExecutionResultBody, GDEXError> {
            let request = UpdateTimeRequest::new(latest_time);
            let transaction = Transaction::new(
                self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        fn update_prices(&self, latest_prices: Vec<u64>) -> Result<ExecutionResultBody, GDEXError> {
            let request = UpdatePricesRequest::new(latest_prices);
            let transaction = Transaction::new(
                self.admin_key.public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        fn account_deposit(&self, quantity: u64, sender: AccountPubKey) -> Result<ExecutionResultBody, GDEXError> {
            let request = AccountDepositRequest::new(
                quantity.try_into().map_err(|_| GDEXError::Conversion)?,
                self.admin_key.public(),
            );
            let transaction = Transaction::new(&sender, CertificateDigest::new([0; fastcrypto::DIGEST_LEN]), &request);
            self.controller_router.handle_consensus_transaction(&transaction)
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

        pub fn futures_limit_order(
            &self,
            user_index: usize,
            side: u64,
            price: u64,
            quantity: u64,
        ) -> Result<ExecutionResultBody, GDEXError> {
            let request = FuturesLimitOrderRequest::new(
                self.base_asset_id,
                self.quote_asset_id,
                side,
                price,
                quantity,
                self.admin_key.public(),
            );

            let transaction = Transaction::new(
                self.user_keys[user_index].public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        pub fn cancel_open_orders(
            &self,
            sender_index: usize,
            target_index: usize,
        ) -> Result<ExecutionResultBody, GDEXError> {
            let request = CancelAllRequest::new(self.user_keys[target_index].public(), self.admin_key.public());

            let transaction = Transaction::new(
                self.user_keys[sender_index].public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        pub fn liquidate(
            &self,
            sender_index: usize,
            target_index: usize,
            side: u64,
            quantity: u64,
        ) -> Result<ExecutionResultBody, GDEXError> {
            let request = LiquidateRequest::new(
                self.base_asset_id,
                self.quote_asset_id,
                side,
                quantity,
                self.user_keys[target_index].public(),
                self.admin_key.public(),
            );

            let transaction = Transaction::new(
                self.user_keys[sender_index].public(),
                CertificateDigest::new([0; fastcrypto::DIGEST_LEN]),
                &request,
            );
            self.controller_router.handle_consensus_transaction(&transaction)
        }

        // UTIILITY FUNCTIONS
        fn get_user_total_req_collateral(&self, user_index: usize) -> Result<u64, GDEXError> {
            self.controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_total_req_collateral(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_user_account_value(&self, user_index: usize) -> Result<i64, GDEXError> {
            self.controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_value(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_user_state_by_market(&self, user_index: usize) -> Result<AccountStateByMarket, GDEXError> {
            self.controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_state_by_market(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_user_unrealized_pnl(&self, user_index: usize) -> Result<i64, GDEXError> {
            self.controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_unrealized_pnl(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_account_available_deposit(&self, user_index: usize) -> Result<i64, GDEXError> {
            self.controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_available_deposit(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_account_deposit(&self, user_index: usize) -> Result<i64, GDEXError> {
            self.controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_deposit(self.admin_key.public(), self.user_keys[user_index].public())
        }

        fn get_account_position(&self, user_index: usize) -> Result<FuturesPosition, GDEXError> {
            let account_state = self
                .controller_router
                .futures_controller
                .lock()
                .unwrap()
                .get_account_state_by_market(self.admin_key.public(), self.user_keys[user_index].public())?;
            let position = account_state.get(0).unwrap().2.as_ref().unwrap();
            Ok(position.clone())
        }
    }

    impl ControllerTestBed for FuturesControllerTester {
        fn get_controller_router(&self) -> &ControllerRouter {
            &self.controller_router
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
            .get_user_state_by_market(maker_index)
            .unwrap()
            .pop()
            .unwrap()
            .2
            .unwrap();
        let taker_position = futures_tester
            .get_user_state_by_market(taker_index)
            .unwrap()
            .pop()
            .unwrap()
            .2
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
        assert!(maker_unrealized_pnl == -taker_unrealized_pnl);
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
    fn liquidate_long_full() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Bid as u64, 21_000_000, 961);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Ask as u64, 21_000_000, 960);

        let maker_init_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_init_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_init_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let init_is_liquidatable = maker_init_deposit + maker_init_unrealized_pnl < 0_i64;
        assert!(!init_is_liquidatable);
        assert_eq!(maker_init_unrealized_pnl, 0);

        let liquidate_result = futures_tester.liquidate(taker_index, maker_index, OrderSide::Bid as u64, 960);
        assert!(liquidate_result.is_err());

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        let maker_order_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_order_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_order_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let order_is_liquidatable =
            maker_order_deposit + maker_order_unrealized_pnl < 0_i64;

        assert!(!order_is_liquidatable);
        assert_eq!(maker_order_unrealized_pnl, 0);
        assert!(maker_order_req_collateral > maker_init_req_collateral); // open orders require collateral

        let mut cancel_result = futures_tester.cancel_open_orders(taker_index, maker_index);
        assert!(cancel_result.unwrap_err() == GDEXError::OrderRequest);

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        let maker_fill_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_fill_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_fill_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let fill_is_liquidatable = maker_fill_deposit + maker_fill_unrealized_pnl < 0_i64;

        assert!(fill_is_liquidatable); // oracle price is 11_000_000, filled at 21_000_000, immediate loss -> liquidation
        assert_eq!(maker_fill_unrealized_pnl, -960 * 10_000_000);

        cancel_result = futures_tester.cancel_open_orders(taker_index, maker_index);
        assert!(cancel_result.is_ok());
        let maker_cancel_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_cancel_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_cancel_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();

        let cancel_is_liquidatable =
            maker_cancel_deposit + maker_cancel_unrealized_pnl < 0_i64;
        assert!(cancel_is_liquidatable); // should still be able to liquidate since the remaining order is small
        assert!(maker_cancel_req_collateral < maker_fill_req_collateral); // cancelling open orders should reduce collateral requirement

        // try to liquidate taker for good measure
        let taker_liquidate_result = futures_tester.liquidate(maker_index, taker_index, OrderSide::Ask as u64, 1);
        assert!(taker_liquidate_result.is_err());

        let liquidate_result = futures_tester.liquidate(taker_index, maker_index, OrderSide::Bid as u64, 960);
        assert!(liquidate_result.is_ok());

        let maker_final_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_final_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_final_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let final_is_liquidatable =
            maker_final_deposit + maker_final_unrealized_pnl < 0_i64;

        assert!(!final_is_liquidatable);
        assert!(maker_final_req_collateral == 1); // zero position rounded up
        assert!(maker_final_unrealized_pnl == 0); // all pnl realized
        assert!(
            maker_final_deposit
                == maker_init_deposit - 960 * (21_000_000 - (INITIAL_ASSET_PRICES[0] as f64 * 0.99) as i64)
        );
    }

    #[test]
    fn liquidate_long_partial() {
        let futures_tester = FuturesControllerTester::new();
        futures_tester.initialize();
        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Bid as u64, 21_000_000, 961);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Ask as u64, 21_000_000, 960);

        let maker_init_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_init_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_init_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let init_is_liquidatable = maker_init_deposit + maker_init_unrealized_pnl < 0_i64;
        assert!(!init_is_liquidatable);
        assert_eq!(maker_init_unrealized_pnl, 0);

        let liquidate_result = futures_tester.liquidate(taker_index, maker_index, OrderSide::Bid as u64, 960);
        assert!(liquidate_result.is_err());

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        let maker_order_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_order_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_order_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let order_is_liquidatable =
            maker_order_deposit + maker_order_unrealized_pnl < 0_i64;

        assert!(!order_is_liquidatable);
        assert_eq!(maker_order_unrealized_pnl, 0);
        assert!(maker_order_req_collateral > maker_init_req_collateral); // open orders require collateral

        let mut cancel_result = futures_tester.cancel_open_orders(taker_index, maker_index);
        assert!(cancel_result.unwrap_err() == GDEXError::OrderRequest);

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        let maker_fill_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_fill_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_fill_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let fill_is_liquidatable = maker_fill_deposit + maker_fill_unrealized_pnl < 0_i64;

        assert!(fill_is_liquidatable); // oracle price is 11_000_000, filled at 21_000_000, immediate loss -> liquidation
        assert_eq!(maker_fill_unrealized_pnl, -960 * 10_000_000);

        cancel_result = futures_tester.cancel_open_orders(taker_index, maker_index);
        assert!(cancel_result.is_ok());
        let maker_cancel_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_cancel_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_cancel_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();

        let cancel_is_liquidatable =
            maker_cancel_deposit + maker_cancel_unrealized_pnl < 0_i64;
        assert!(cancel_is_liquidatable); // should still be able to liquidate since the remaining order is small
        assert!(maker_cancel_req_collateral < maker_fill_req_collateral); // cancelling open orders should reduce collateral requirement

        let liquidate_result = futures_tester.liquidate(taker_index, maker_index, OrderSide::Bid as u64, 50);
        assert!(liquidate_result.is_ok());

        let maker_partial_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_partial_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let partial_is_liquidatable =
            maker_partial_deposit + maker_partial_unrealized_pnl < 0_i64;

        // partial liquidation, target still liquidatable
        assert!(partial_is_liquidatable);
        assert!(maker_partial_unrealized_pnl == -910 * 10_000_000);
        assert_eq!(
            maker_partial_deposit,
            maker_init_deposit
                - 50 * (21_000_000 - (INITIAL_ASSET_PRICES[0] as f64 * 0.99) as i64)
                - 910 * 11_000_000 / TEST_MAX_LEVERAGE as i64
        );

        let liquidate_result = futures_tester.liquidate(taker_index, maker_index, OrderSide::Bid as u64, 650);
        assert!(liquidate_result.is_ok());

        let maker_final_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let maker_final_req_collateral = futures_tester.get_user_total_req_collateral(maker_index).unwrap();
        let maker_final_unrealized_pnl = futures_tester.get_user_unrealized_pnl(maker_index).unwrap();
        let final_is_liquidatable =
            maker_final_deposit + maker_final_unrealized_pnl < 0_i64;

        assert!(!final_is_liquidatable);

        assert!(maker_final_req_collateral == 260 * 11_000_000 / TEST_MAX_LEVERAGE as u64 + 1);
        assert!(maker_final_unrealized_pnl == -260 * 10_000_000);
        assert_eq!(
            maker_final_deposit,
            maker_init_deposit
                - 700 * (21_000_000 - (INITIAL_ASSET_PRICES[0] as f64 * 0.99) as i64)
                - maker_final_req_collateral as i64
                + 1
        );
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

        // TODO - https://github.com/gdexorg/gdex/issues/166 - the assert below works, but residual rounding from price calculation has been added back in
        // can we easily calculate this dynamically?
        // e.g. in this example the residual is floor(1975 * .886...) = 70, where
        // the residual .886 comes from the rounding on price = quantity_1 * price_1 + quantity_2 * price_2 / quantity_1 + quantity_2
        let user_req_collateral_post_cross = futures_tester.get_user_total_req_collateral(user_index).unwrap();

        assert!(
            user_req_collateral
                == user_req_collateral_post_cross + (user_price_1 * user_quantity_3) / TEST_MAX_LEVERAGE + 70
        );

        // ensure that we don't have a remainder transaction
        let user_position = futures_tester
            .get_user_state_by_market(user_index)
            .unwrap()
            .pop()
            .unwrap()
            .2;

        assert!(user_position.is_none())
    }

    // TODO liquidation tests for short, multi order

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

        let maker_position = futures_tester
            .get_user_state_by_market(maker_index)
            .unwrap()
            .pop()
            .unwrap()
            .2;
        let taker_position = futures_tester
            .get_user_state_by_market(taker_index)
            .unwrap()
            .pop()
            .unwrap()
            .2;

        assert!(maker_position.is_none());
        assert!(taker_position.is_none());

        let maker_available_deposit = futures_tester.get_account_available_deposit(maker_index).unwrap();
        let taker_available_deposit = futures_tester.get_account_available_deposit(taker_index).unwrap();

        assert!(maker_available_deposit as u64 == USER_INITIAL_DEPOSIT + 1_000_000 * 10 - 1);
        assert!(taker_available_deposit as u64 == USER_INITIAL_DEPOSIT - 1_000_000 * 10 - 1);
    }
}
