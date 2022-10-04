// IMPORTS

// crate

// local
use crate::futures::types::{FuturesUserByMarket, MarketResponse, MarketplaceResponse, MarketplaceUserInfoResponse};
use crate::router::ControllerRouter;
use gdex_types::asset::AssetId;
use gdex_types::order_book::OrderbookDepth;
use gdex_types::store::RPCStoreHandle;
use gdex_types::{account::AccountPubKey, crypto::ToFromBytes, utils};

// mysten
use sui_json_rpc::SuiRpcModule;
use sui_open_rpc::Module;
use sui_open_rpc_macros::open_rpc;

// external
use jsonrpsee::core::{async_trait, Error, RpcResult};
use jsonrpsee::RpcModule;
use jsonrpsee_proc_macros::rpc;
use std::sync::{Arc, Mutex};

// To implement a custom RPC server, one starts with a trait that defines RPC methods
// The trait must be annotated with the `rpc` decorator.
// The methods must return a `Result` type.
// The `#[method(name = "foo")]` attribute is optional and results in a method named `{namespace}_foo`.
// If the attribute is not present, the method name is `{namespace}_{method_name}`.
#[open_rpc(namespace = "tenex", tag = "Primary RPC API")]
#[rpc(server, client, namespace = "tenex")]
pub trait ControllerData {
    #[method(name = "getFuturesMarketplaces")]
    async fn get_market_places(&self) -> RpcResult<Vec<MarketplaceResponse>>;
    #[method(name = "getMarkets")]
    async fn get_markets(&self, market_admin: String) -> RpcResult<Vec<MarketResponse>>;
    #[method(name = "getUserMarketplaceInfo")]
    async fn get_user_marketplace_info(
        &self,
        market_admin: String,
        user: String,
    ) -> RpcResult<MarketplaceUserInfoResponse>;
    #[method(name = "getOrderbookDepth")]
    async fn get_orderbook_depth(
        &self,
        market_admin: String,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        depth: usize, // max depth of 100
    ) -> RpcResult<OrderbookDepth>;
}

// The JSONRPCService struct will implement the RPC server
// To do so, it must implement the trait `{TraitName}Server`
// this trait is generated by the rpc method above
// TODO -  - use RWLock
pub struct JSONRPCService {
    state_manager: Arc<Mutex<ControllerRouter>>,
    rpc_store_handle: Arc<RPCStoreHandle>,
}

impl JSONRPCService {
    #[allow(clippy::new_without_default)]
    pub fn new(state_manager: Arc<Mutex<ControllerRouter>>, rpc_store_handle: Arc<RPCStoreHandle>) -> Self {
        Self {
            state_manager,
            rpc_store_handle,
        }
    }
}

#[async_trait]
impl ControllerDataServer for JSONRPCService {
    async fn get_market_places(&self) -> RpcResult<Vec<MarketplaceResponse>> {
        let locked_state_manager = self.state_manager.lock().unwrap();
        let locked_futures_market = locked_state_manager.futures_controller.lock().unwrap();
        let market_places = locked_futures_market.get_marketplaces();

        // unpack marketplaces from marketplaces container into a vector MarketplaceResponses
        let mut market_places_vec = Vec::new();
        for (address, market_place) in market_places {
            let supported_base_asset_ids: Vec<u64> = market_place.markets.keys().cloned().collect();
            market_places_vec.push(MarketplaceResponse {
                admin: utils::encode_bytes_hex(address),
                quote_asset_id: market_place.quote_asset_id,
                supported_base_asset_ids,
            });
        }

        Ok(market_places_vec)
    }

    async fn get_markets(&self, market_admin: String) -> RpcResult<Vec<MarketResponse>> {
        let locked_state_manager = self.state_manager.lock().unwrap();
        let locked_futures_market = locked_state_manager.futures_controller.lock().unwrap();

        let account_bytes: Vec<u8> = utils::decode_bytes_hex(&market_admin)?;
        let market_admin: AccountPubKey = AccountPubKey::from_bytes(account_bytes.as_slice())
            .map_err(|_| Error::Custom("Failed to decode market admin".to_string()))?;
        let market_place = locked_futures_market
            .get_marketplaces()
            .get(&market_admin)
            .ok_or_else(|| Error::Custom("Failed to load marketplace for admin".to_string()))
            .unwrap();

        // unpack markets from market place into a vector of MarketResponses
        let mut markets_vec = Vec::new();
        for (asset, market) in market_place.markets.iter() {
            markets_vec.push(MarketResponse {
                max_leverage: market.max_leverage,
                base_asset_id: *asset,
                quote_asset_id: market_place.quote_asset_id,
                open_interest: market.open_interest,
                last_traded_price: market.last_traded_price,
                oracle_price: market.oracle_price,
            });
        }
        Ok(markets_vec)
    }

    async fn get_user_marketplace_info(
        &self,
        market_admin: String,
        user: String,
    ) -> RpcResult<MarketplaceUserInfoResponse> {
        let locked_state_manager = self.state_manager.lock().unwrap();
        let locked_futures_market = locked_state_manager.futures_controller.lock().unwrap();

        let market_admin_bytes: Vec<u8> = utils::decode_bytes_hex(&market_admin)?;
        let market_admin: AccountPubKey = AccountPubKey::from_bytes(market_admin_bytes.as_slice())
            .map_err(|_| Error::Custom("Failed to decode market admin".to_string()))?;

        let user_bytes: Vec<u8> = utils::decode_bytes_hex(&user)?;
        let user: AccountPubKey = AccountPubKey::from_bytes(user_bytes.as_slice())
            .map_err(|_| Error::Custom("Failed to decode market admin".to_string()))?;

        let market_place = locked_futures_market
            .get_marketplaces()
            .get(&market_admin)
            .ok_or_else(|| Error::Custom("Failed to load marketplace for admin".to_string()))
            .unwrap();

        let user_deposit = locked_futures_market
            .get_account_deposit(&market_admin, &user)
            .map_err(|_| Error::Custom("Could not load user deposit".to_string()))?;

        let user_collateral_req = locked_futures_market
            .get_account_total_req_collateral(&market_admin, &user)
            .map_err(|_| Error::Custom("Could not load user required collateral".to_string()))?;

        let user_unrealized_pnl = locked_futures_market
            .get_account_unrealized_pnl(&market_admin, &user)
            .map_err(|_| Error::Custom("Could not load user deposit".to_string()))?;

        let user_market_info: Vec<FuturesUserByMarket> = locked_futures_market
            .get_account_state_by_market(&market_admin, &user)
            .map_err(|_| Error::Custom("Could not load position data".to_string()))?
            .iter()
            .map(|account_state| FuturesUserByMarket {
                base_asset_id: account_state.0,
                orders: account_state.1.clone(),
                position: account_state.2.clone(),
            })
            .collect();
        Ok(MarketplaceUserInfoResponse {
            user_deposit,
            user_collateral_req,
            user_unrealized_pnl,
            user_market_info,
            quote_asset_id: market_place.quote_asset_id,
        })
    }

    async fn get_orderbook_depth(
        &self,
        market_admin: String,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        depth: usize,
    ) -> RpcResult<OrderbookDepth> {
        // do not allow large snapshots to be returned
        if depth > 100 {
            return Err(Error::Custom("Depth exceeds 100 which is not allowed".to_string()));
        }

        let market_admin_bytes: Vec<u8> = utils::decode_bytes_hex(&market_admin)?;
        let market_admin: AccountPubKey = AccountPubKey::from_bytes(market_admin_bytes.as_slice())
            .map_err(|_| Error::Custom("Failed to decode market admin".to_string()))?;

        let orderbook_depth_key = crate::futures::controller::FuturesController::get_orderbook_key(
            &market_admin,
            base_asset_id,
            quote_asset_id,
        );

        let mut return_value = self
            .rpc_store_handle
            .rpc_store
            .latest_orderbook_depth_store
            .read(orderbook_depth_key)
            .await
            .map_err(|_| Error::Custom("Could not load orderbook depth".to_string()))?
            .ok_or_else(|| Error::Custom("Orderbook is empty".to_string()))?;

        // reduce each depth level to the requested depth
        return_value = OrderbookDepth {
            bids: return_value.bids.into_iter().rev().take(depth).collect(),
            asks: return_value.asks.into_iter().take(depth).collect(),
        };

        Ok(return_value)
    }
}

// The SuiRPCModule allows us to generate an OpenRPC document for the RPC server.
impl SuiRpcModule for JSONRPCService {
    fn rpc(self) -> RpcModule<Self> {
        ControllerDataServer::into_rpc(self)
    }

    fn rpc_doc_module() -> Module {
        crate::bank::rpc_server::ControllerDataOpenRpc::module_doc()
    }
}
