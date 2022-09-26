// IMPORTS

// crate
use crate::router::ControllerType;

// gdex
use gdex_types::{
    account::AccountPubKey,
    error::GDEXError,
    transaction::{Request, RequestTypeEnum},
};

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

// ENUM

impl RequestTypeEnum for FuturesRequestType {
    fn request_type_from_i32(value: i32) -> Result<Self, GDEXError> {
        match value {
            0 => Ok(FuturesRequestType::CreateMarketplace),
            1 => Ok(FuturesRequestType::CreateMarket),
            2 => Ok(FuturesRequestType::UpdateMarketParams),
            3 => Ok(FuturesRequestType::UpdateTime),
            4 => Ok(FuturesRequestType::UpdatePrices),
            5 => Ok(FuturesRequestType::AccountDeposit),
            6 => Ok(FuturesRequestType::AccountWithdrawal),
            7 => Ok(FuturesRequestType::FuturesLimitOrder),
            8 => Ok(FuturesRequestType::CancelOrder),
            9 => Ok(FuturesRequestType::CancelAll),
            10 => Ok(FuturesRequestType::Liquidate),
            _ => Err(GDEXError::DeserializationError),
        }
    }
}

// INTERFACE

// create marketplace

impl CreateMarketplaceRequest {
    pub fn new(quote_asset_id: u64) -> Self {
        CreateMarketplaceRequest { quote_asset_id }
    }
}

impl Request for CreateMarketplaceRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::CreateMarketplace as i32
    }
}

// create market

impl CreateMarketRequest {
    pub fn new(base_asset_id: u64) -> Self {
        CreateMarketRequest { base_asset_id }
    }
}

impl Request for CreateMarketRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::CreateMarket as i32
    }
}

// update market params

impl UpdateMarketParamsRequest {
    pub fn new(base_asset_id: u64, max_leverage: u64) -> Self {
        UpdateMarketParamsRequest {
            base_asset_id,
            max_leverage,
        }
    }
}

impl Request for UpdateMarketParamsRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::UpdateMarketParams as i32
    }
}

// update time

impl UpdateTimeRequest {
    pub fn new(latest_time: u64) -> Self {
        UpdateTimeRequest { latest_time }
    }
}

impl Request for UpdateTimeRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::UpdateTime as i32
    }
}

// update prices

impl UpdatePricesRequest {
    pub fn new(latest_prices: Vec<u64>) -> Self {
        UpdatePricesRequest { latest_prices }
    }
}

impl Request for UpdatePricesRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::UpdatePrices as i32
    }
}

// account deposit

// TODO should we use u64 here rather than i64
impl AccountDepositRequest {
    pub fn new(quantity: i64, market_admin: &AccountPubKey) -> Self {
        AccountDepositRequest {
            quantity,
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
        }
    }
}

impl Request for AccountDepositRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::AccountDeposit as i32
    }
}

// account withdrawal

impl AccountWithdrawalRequest {
    pub fn new(quantity: u64, market_admin: &AccountPubKey) -> Self {
        AccountWithdrawalRequest {
            quantity,
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
        }
    }
}

impl Request for AccountWithdrawalRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::AccountWithdrawal as i32
    }
}

// futures limit order

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

impl CancelAllRequest {
    pub fn new(target: &AccountPubKey, market_admin: &AccountPubKey) -> Self {
        CancelAllRequest {
            target: Bytes::from(target.as_ref().to_vec()),
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
        }
    }
}

impl LiquidateRequest {
    pub fn new(
        base_asset_id: u64,
        quote_asset_id: u64,
        side: u64,
        quantity: u64,
        market_admin: &AccountPubKey,
        target: &AccountPubKey,
    ) -> Self {
        LiquidateRequest {
            base_asset_id,
            quote_asset_id,
            side,
            quantity,
            market_admin: Bytes::from(market_admin.as_ref().to_vec()),
            target: Bytes::from(target.as_ref().to_vec()),
        }
    }
}

impl Request for LiquidateRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::Liquidate as i32
    }
}

impl Request for CancelAllRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::CancelAll as i32
    }
}

impl Request for FuturesLimitOrderRequest {
    fn get_controller_id() -> i32 {
        ControllerType::Futures as i32
    }
    fn get_request_type_id() -> i32 {
        FuturesRequestType::FuturesLimitOrder as i32
    }
}

/// Begin externally available testing functions
#[cfg(any(test, feature = "testing"))]
pub mod futures_controller_test_functions {
    use super::*;
    use fastcrypto::DIGEST_LEN;
    use gdex_types::crypto::ToFromBytes;
    use gdex_types::transaction::Transaction;
    use gdex_types::{account::AccountKeyPair, crypto::KeypairTraits, transaction::SignedTransaction};
    use narwhal_types::CertificateDigest;

    pub const PRIMARY_ASSET_ID: u64 = 0;

    pub fn generate_signed_limit_order(
        kp_sender: &AccountKeyPair,
        kp_admin: &AccountKeyPair,
        base_asset_id: u64,
        quote_asset_id: u64,
        side: u64,
        price: u64,
        quantity: u64,
    ) -> SignedTransaction {
        // TODO replace this with latest

        let request = FuturesLimitOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
            market_admin: bytes::Bytes::from(kp_admin.public().as_bytes().to_vec()),
        };

        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);
        let transaction = Transaction::new(kp_sender.public(), dummy_batch_digest, &request);

        transaction.sign(kp_sender).unwrap()
    }
}
