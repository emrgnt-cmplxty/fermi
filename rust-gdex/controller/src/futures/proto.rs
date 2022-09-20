// IMPORTS

// gdex
use gdex_types::account::AccountPubKey;

// external
use prost::bytes::Bytes;

// MODULE IMPORTS

#[path = "./generated/futures_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod futures_requests;

pub use futures_requests::*;

// HELPER

use crate::spot::proto::LimitOrderRequest; // TODO bad, controllers should not depend on eachother like this
impl From<FuturesLimitOrderRequest> for LimitOrderRequest {
    fn from(request: FuturesLimitOrderRequest) -> Self {
        Self {
            base_asset_id: request.base_asset_id,
            quote_asset_id: request.quote_asset_id,
            side: request.side,
            price: request.price,
            quantity: request.quantity,
        }
    }
}

// INTERFACE

impl CreateMarketplaceRequest {
    pub fn new(quote_asset_id: u64) -> Self {
        CreateMarketplaceRequest { quote_asset_id }
    }
}

impl CreateMarketRequest {
    pub fn new(base_asset_id: u64) -> Self {
        CreateMarketRequest { base_asset_id }
    }
}

impl UpdateMarketParamsRequest {
    pub fn new(base_asset_id: u64, max_leverage: u64) -> Self {
        UpdateMarketParamsRequest {
            base_asset_id,
            max_leverage,
        }
    }
}

impl UpdateTimeRequest {
    pub fn new(latest_time: u64) -> Self {
        UpdateTimeRequest { latest_time }
    }
}

impl UpdatePricesRequest {
    pub fn new(latest_prices: Vec<u64>) -> Self {
        UpdatePricesRequest { latest_prices }
    }
}

// TODO should we use u64 here rather than i64
impl AccountDepositRequest {
    pub fn new(quantity: i64, market_admin: &AccountPubKey) -> Self {
        AccountDepositRequest {
            quantity,
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
        }
    }
}

impl AccountWithdrawalRequest {
    pub fn new(quantity: u64, market_admin: &AccountPubKey) -> Self {
        AccountWithdrawalRequest {
            quantity,
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
        }
    }
}

impl FuturesLimitOrderRequest {
    pub fn new(
        base_asset_id: u64,
        quote_asset_id: u64,
        side: u64,
        price: u64,
        quantity: u64,
        market_admin: &AccountPubKey,
    ) -> Self {
        FuturesLimitOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
        }
    }
}
