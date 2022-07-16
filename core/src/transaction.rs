extern crate engine;

use std::time;

use diem_crypto::hash::CryptoHash;
use engine::orders::{OrderRequest, new_limit_order_request};
use types::account::{AccountPubKey, AccountPrivKey};
use types::orderbook::{OrderSide};
// make a new struct for an order that we have to hash
pub struct Order
{
    order: OrderRequest,
    addr: AccountPubKey,
    sig: AccountPrivKey,
}

pub struct Payment
{
    payment: OrderRequest,
    addr: AccountPubKey,
    sig: AccountPrivKey,
}

pub struct CreateAsset
{
    payment: OrderRequest,
    addr: AccountPubKey,
    sig: AccountPrivKey,
}
pub enum Transaction
{
    OrderTransaction(Order),
    PaymentTransaction(Payment),
    CreateAssetTransaction(Payment)
}


#[test]
fn transaction_test() {

    let base_asset = 0;
    let quote_asset = 1;

    let price = 1;
    let qty = 10;

    let order: OrderRequest= new_limit_order_request(
        base_asset,
        quote_asset,
        OrderSide::Bid,
        price,
        qty,
        time::SystemTime::now()
    );
    order.hash();
}
