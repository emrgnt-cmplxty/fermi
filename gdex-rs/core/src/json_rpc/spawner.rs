// gdex
use crate::client::endpoint_from_multiaddr;
use crate::{json_rpc::processor::BlockProcessor, validator::genesis_state::ValidatorGenesisState};
use gdex_controller::router::ControllerRouter;
use gdex_types::{
    proto::ValidatorGrpcClient,
    store::{RPCStore, RPCStoreHandle}
};
// mysten
use sui_json_rpc::SuiRpcModule;
// external
use jsonrpsee::{http_server::HttpServerBuilder, RpcModule};
use multiaddr::Multiaddr;
use std::{
    path::PathBuf,
    sync::{atomic::AtomicU64, Arc, Mutex},
};
use tokio::task::JoinHandle;
use tracing::info;

// INTERFACE

pub struct JSONServiceSpawner {
    controller_router: Arc<Mutex<ControllerRouter>>,
    grpc_addr: Multiaddr,
    jsonrpc_addr: Multiaddr,
    latest_listened_block: Arc<AtomicU64>,
    rpc_store_handle: Arc<RPCStoreHandle>,
}

impl JSONServiceSpawner {
    pub fn new(
        genesis_state: ValidatorGenesisState,
        grpc_addr: Multiaddr,
        jsonrpc_addr: Multiaddr,
        json_rpc_db: PathBuf,
    ) -> Self {
        let controller_router = Arc::new(Mutex::new(genesis_state.controller_router().clone()));
        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(json_rpc_db),
        });

        Self {
            controller_router,
            grpc_addr,
            jsonrpc_addr,
            latest_listened_block: Arc::new(AtomicU64::new(0)),
            rpc_store_handle,
        }
    }

    pub async fn construct_rpc_module(
        controller_router: Arc<Mutex<ControllerRouter>>,
        rpc_store_handle: Arc<RPCStoreHandle>,
        grpc_addr: &Multiaddr,
    ) -> RpcModule<()> {
        let mut module = ControllerRouter::generate_rpc_module(controller_router, rpc_store_handle);
        let grpc_endpoint = endpoint_from_multiaddr(grpc_addr).unwrap();
        let grpc_client = ValidatorGrpcClient::connect(grpc_endpoint.endpoint().clone())
            .await
            .unwrap();

        module
            .merge(crate::json_rpc::server::JSONRPCService::new(grpc_client).rpc())
            .unwrap();

        module
    }

    pub async fn spawn_jsonrpc_service(&mut self) -> anyhow::Result<Vec<JoinHandle<()>>> {
        let mut results = Vec::new();
        // block listener
        let block_processor_handles = BlockProcessor::spawn(
            self.controller_router.clone(),
            Arc::clone(&self.latest_listened_block),
            Arc::clone(&self.rpc_store_handle),
            self.grpc_addr.clone(),
        );

        // launch block listener task
        results.extend(block_processor_handles);

        info!("Spawning a JSON RPC with address = {}", self.jsonrpc_addr);
        // rpc module
        let module = Self::construct_rpc_module(
            self.controller_router.clone(),
            self.rpc_store_handle.clone(),
            &self.grpc_addr,
        )
        .await;

        let socket_addr = crate::multiaddr::to_socket_addr(&self.jsonrpc_addr).unwrap();

        // TODO - the code below is an ugly hack to transport ServerHandle into a JoinHandle<()>
        // How can we more intelligently clean this up?
        let handle = tokio::spawn(async move {
            let server = HttpServerBuilder::default().build(&socket_addr).await.unwrap();
            let _handle = server.start(module).unwrap();
            // sleep for a loooooong time
            tokio::time::sleep(tokio::time::Duration::from_secs(10_000_000_000_000_000)).await;
        });
        // spawn rpc server
        results.push(handle);
        Ok(results)
    }
}

#[cfg(test)]
mod test_json_rpc_server {
    use gdex_controller::futures::test::futures_tests::FuturesControllerTester;
    use gdex_controller::futures::types::{MarketResponse, MarketplaceResponse, MarketplaceUserInfoResponse};
    use gdex_controller::router::ControllerRouter;
    use gdex_controller::ControllerTestBed;
    use gdex_types::crypto::KeypairTraits;
    use gdex_types::order_book::{OrderSide, OrderbookDepth};
    use gdex_types::{
        account::{account_test_functions::generate_keypair_vec, AccountPubKey},
        crypto::ToFromBytes,
        utils,
    };

    use crate::json_rpc::spawner::{RPCStore, RPCStoreHandle};
    use jsonrpsee::http_client::HttpClientBuilder;
    use jsonrpsee::http_server::{HttpServerBuilder, HttpServerHandle};
    use jsonrpsee::rpc_params;
    use jsonrpsee_core::client::ClientT;
    use std::{
        net::SocketAddr,
        sync::{Arc, Mutex},
    };

    fn local_controller_router(intitializer: &AccountPubKey) -> Arc<Mutex<ControllerRouter>> {
        let controller_router = Arc::new(Mutex::new(ControllerRouter::default()));
        {
            let controller_lock = controller_router.lock().unwrap();

            controller_lock.initialize_controllers();
            controller_lock.initialize_controller_accounts();

            controller_lock
                .bank_controller
                .lock()
                .unwrap()
                .create_asset(intitializer)
                .unwrap();
        }
        controller_router
    }

    async fn run_server(
        controller_router: Arc<Mutex<ControllerRouter>>,
        rpc_store_handle: Arc<RPCStoreHandle>,
    ) -> anyhow::Result<(SocketAddr, HttpServerHandle)> {
        let new_port = utils::available_local_socket_address();
        let server = HttpServerBuilder::default().build(new_port).await?;
        let addr = server.local_addr()?;
        // initialize a controller router and consume to produce a JSONRPCService
        let module = ControllerRouter::generate_rpc_module(controller_router, rpc_store_handle);

        let server_handle = server.start(module)?; //.into_rpc())?;

        Ok((addr, server_handle))
    }

    // TEST BANK ENDPOINTS

    #[tokio::test]
    async fn test_get_account_balance() -> anyhow::Result<()> {
        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        let controller_router = local_controller_router(intitializer.public());

        let rpc_temp_dir = tempfile::tempdir().unwrap();
        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(rpc_temp_dir),
        });
        let (server_addr, _handle) = run_server(controller_router, rpc_store_handle).await?;
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url)?;

        let params = rpc_params![
            /* user */ utils::encode_bytes_hex(intitializer.public().as_bytes().to_vec()),
            /* asset */ 0
        ];
        let response: Result<u64, _> = client.request("tenex_getAccountBalance", params).await;
        assert!(response.unwrap() == 10_000_000_000_000_000);
        Ok(())
    }

    // TEST FUTURES ENDPOINTS

    #[tokio::test]
    async fn test_get_market_places() -> anyhow::Result<()> {
        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        let futures_tester = FuturesControllerTester::from_admin(intitializer);
        futures_tester.initialize();

        let rpc_temp_dir = tempfile::tempdir().unwrap();
        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(rpc_temp_dir),
        });
        let (server_addr, _handle) = run_server(futures_tester.get_controller_router(), rpc_store_handle).await?;
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url)?;

        let params = None;
        let response: Result<Vec<MarketplaceResponse>, _> =
            client.request("tenex_getFuturesMarketplaces", params).await;
        let expected_marketplace = response.unwrap().pop().unwrap();
        assert!(expected_marketplace.quote_asset_id == 1);
        assert!(expected_marketplace.supported_base_asset_ids == vec![0]);
        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        assert!(expected_marketplace.admin == utils::encode_bytes_hex(intitializer.public().as_bytes().to_vec()));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_markets() -> anyhow::Result<()> {
        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        let futures_tester = FuturesControllerTester::from_admin(intitializer);
        futures_tester.initialize();

        let rpc_temp_dir = tempfile::tempdir().unwrap();
        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(rpc_temp_dir),
        });
        let (server_addr, _handle) = run_server(futures_tester.get_controller_router(), rpc_store_handle).await?;
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url)?;

        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();

        let params = rpc_params![/* market_admin */ utils::encode_bytes_hex(intitializer.public().as_bytes().to_vec())];
        let response: Result<Vec<MarketResponse>, _> = client.request("tenex_getMarkets", params).await;
        let expected_market = response.unwrap().pop().unwrap();

        assert!(expected_market.max_leverage == gdex_controller::futures::test::futures_tests::TEST_MAX_LEVERAGE);
        assert!(expected_market.base_asset_id == gdex_controller::futures::test::futures_tests::BASE_ASSET_ID);
        assert!(expected_market.quote_asset_id == gdex_controller::futures::test::futures_tests::QUOTE_ASSET_ID);
        assert!(expected_market.open_interest == 0);
        assert!(expected_market.last_traded_price == 0);
        assert!(expected_market.oracle_price == gdex_controller::futures::test::futures_tests::INITIAL_ASSET_PRICES[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_marketplace_info() -> anyhow::Result<()> {
        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        let futures_tester = FuturesControllerTester::from_admin(intitializer);
        futures_tester.initialize();

        // cross the spread to create a position for our user
        let (maker_index, maker_side, maker_price, maker_quantity) = (0, OrderSide::Bid as u64, 10_000_000, 100);
        // taker must ask less than maker price in order to be a taker
        let (taker_index, taker_side, taker_price, taker_quantity) = (1, OrderSide::Ask as u64, 10_000_000 - 1, 10);

        futures_tester
            .futures_limit_order(maker_index, maker_side, maker_price, maker_quantity)
            .unwrap();

        futures_tester
            .futures_limit_order(taker_index, taker_side, taker_price, taker_quantity)
            .unwrap();

        let rpc_temp_dir = tempfile::tempdir().unwrap();
        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(rpc_temp_dir),
        });
        let (server_addr, _handle) = run_server(futures_tester.get_controller_router(), rpc_store_handle).await?;
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url)?;

        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        let user_address = futures_tester.user_keys[maker_index].public();

        let params = rpc_params![
            /* market_admin */ utils::encode_bytes_hex(intitializer.public().as_bytes().to_vec()),
            /* user */ utils::encode_bytes_hex(user_address.as_bytes().to_vec())
        ];
        let response: Result<MarketplaceUserInfoResponse, _> =
            client.request("tenex_getUserMarketplaceInfo", params).await;
        let response = response.unwrap();
        assert!(
            response.user_collateral_req
                == (maker_quantity - taker_quantity) * maker_price
                    / gdex_controller::futures::test::futures_tests::TEST_MAX_LEVERAGE
                    + taker_quantity * gdex_controller::futures::test::futures_tests::INITIAL_ASSET_PRICES[0]
                        / gdex_controller::futures::test::futures_tests::TEST_MAX_LEVERAGE
                    + 1
        );
        assert!(
            response.user_unrealized_pnl
                == (gdex_controller::futures::test::futures_tests::INITIAL_ASSET_PRICES[0] - maker_price) as i64
                    * taker_quantity as i64
        );
        Ok(())
    }

    #[tokio::test]
    // TODO - Fill in the rest of the test
    async fn test_get_orderbook_snapshot_info() -> anyhow::Result<()> {
        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();
        let futures_tester = FuturesControllerTester::from_admin(intitializer);
        futures_tester.initialize();

        // for loop with step sizes of 100
        let (max_steps, step_size, quantity) = (100, 100, 100);
        for i in 1..max_steps {
            futures_tester
                .futures_limit_order(0, OrderSide::Bid as u64, i * step_size, quantity)
                .unwrap();
            futures_tester
                .futures_limit_order(
                    0,
                    OrderSide::Ask as u64,
                    step_size * max_steps + i * step_size,
                    quantity,
                )
                .unwrap();
        }

        let rpc_temp_dir = tempfile::tempdir().unwrap();
        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(rpc_temp_dir),
        });
        let (server_addr, _handle) =
            run_server(futures_tester.get_controller_router(), rpc_store_handle.clone()).await?;
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url)?;

        let rpc_store = &rpc_store_handle.rpc_store;

        futures_tester
            .generate_orderbook_depths(
                rpc_store,
                gdex_controller::futures::controller::ORDERBOOK_DEPTH_FREQUENCY,
            )
            .await;

        let intitializer = generate_keypair_vec([0; 32]).pop().unwrap();

        let params = rpc_params![
            /* market_admin */ utils::encode_bytes_hex(intitializer.public().as_bytes().to_vec()),
            /* base_asset_id */ 0,
            /* quote_asset_id */ 1,
            /* depth */ 10
        ];
        let response: Result<OrderbookDepth, _> = client.request("tenex_getOrderbookDepth", params).await;
        let response = response.unwrap();
        assert!(response.bids[0].price == (max_steps - 1) * step_size);
        assert!(response.asks[0].price == (max_steps + 1) * step_size);
        assert!(response.bids[0].quantity == quantity);
        assert!(response.asks[0].quantity == quantity);
        Ok(())
    }
}
