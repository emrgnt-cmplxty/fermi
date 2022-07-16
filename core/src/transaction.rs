extern crate engine;
extern crate proc;

use std::fmt::Debug;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use diem_crypto::{hash::CryptoHash};
use diem_crypto_derive::{CryptoHasher, BCSCryptoHash};
use engine::orders::{new_limit_order_request};
use proc::account::{AccountPubKey, AccountPrivKey};
use types::orderbook::{OrderSide, OrderRequest};

// make a new struct for an order that we have to hash
pub struct Order<Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    order: OrderRequest<Asset>,
    addr: AccountPubKey,
    sig: AccountPrivKey,
}

pub struct Payment<Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    payment: OrderRequest<Asset>,
    addr: AccountPubKey,
    sig: AccountPrivKey,
}

pub struct CreateAsset<Asset> 
where
    Asset: Debug + Clone + Copy + Eq,
{
    payment: OrderRequest<Asset>,
    addr: AccountPubKey,
    sig: AccountPrivKey,
}
pub enum Transaction<Asset: std::fmt::Debug + std::marker::Copy + std::cmp::Eq>
{
    OrderTransaction(Order<Asset>),
    PaymentTransaction(Payment<Asset>),
    CreateAssetTransaction(Payment<Asset>)
}


#[test]
fn transaction_test() {

    #[derive(PartialEq, Eq, Debug, Copy, Clone, Deserialize, Serialize)]
    pub enum BrokerAsset {
        BTC,
        USD,
    }

    let base_asset = BrokerAsset::BTC;
    let quote_asset = BrokerAsset::USD;

    let price = 1;
    let qty = 10;

    let order: OrderRequest<BrokerAsset>= new_limit_order_request(
        base_asset,
        quote_asset,
        OrderSide::Bid,
        price,
        qty,
        SystemTime::now()
    );
    order.hash();
}
