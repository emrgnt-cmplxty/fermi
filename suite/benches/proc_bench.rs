
extern crate engine;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng};
use rand::rngs::{ThreadRng};

use diem_crypto::{
    traits::{SigningKey, Uniform},
};
use engine::orderbook::{Orderbook};
use engine::domain::OrderSide;
use proc::account::{AccountPubKey, AccountPrivKey, AccountSignature, AccountController, TestDiemCrypto, DUMMY_MESSAGE};


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BrokerAsset {
    USD,
    EUR,
    BTC,
    ETH,
}

fn parse_asset(asset: &str) -> Option<BrokerAsset> {
    match asset {
        "USD" => Some(BrokerAsset::USD),
        "EUR" => Some(BrokerAsset::EUR),
        "BTC" => Some(BrokerAsset::BTC),
        "ETH" => Some(BrokerAsset::ETH),
        _ => None,
    }
}

fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i64.pow(decimals) as f64;
    (x * y).round() / y
}

#[inline]
fn place_orders(n_orders: u64, account_to_pub_key: &mut Vec<AccountPubKey>, market_controller: &mut AccountController<BrokerAsset>, rng: &mut ThreadRng) {

    // initialize orderbook
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let mut orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);

    // clean market controller orderbook to keep from getting clogged
    market_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = round(rng.gen_range(0.0..10.0), 3) + 0.001;
        let price = round(rng.gen_range(0.0..10.0), 3) + 0.001;
        // generate a random integer between 0 and 100
        let account_pub_key: AccountPubKey = account_to_pub_key[rng.gen_range(0..100)];

        market_controller.place_limit_order(&account_pub_key,  order_type, qty, price).unwrap();

        i_order+=1;
    }
}

#[inline]
fn place_orders_signed(n_orders: u64, account_to_pub_key: &mut Vec<AccountPubKey>, account_to_signed_msg: &mut Vec<AccountSignature>, market_controller: &mut AccountController<BrokerAsset>, rng: &mut ThreadRng) {

    // initialize orderbook
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let orderbook: Orderbook<BrokerAsset> = Orderbook::new(base_asset, quote_asset);

    // clean market controller orderbook to keep from getting clogged
    market_controller.overwrite_orderbook(orderbook);

    // bench
    let mut i_order: u64 = 0;
    while i_order < n_orders {
        let order_type = if i_order % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
        // generate two random a number between 0.001 and 10 w/ interval of 0.001
        let qty = round(rng.gen_range(0.0..10.0), 3) + 0.001;
        let price = round(rng.gen_range(0.0..10.0), 3) + 0.001;
        // generate a random integer between 0 and 100
        let i_account = rng.gen_range(0..100);
        let account_pub_key: AccountPubKey = account_to_pub_key[i_account];
        let account_sig_msg: &AccountSignature = &account_to_signed_msg[i_account];

        market_controller.place_signed_limit_order(&account_pub_key,  order_type, qty, price, account_sig_msg).unwrap();

        i_order+=1;
    }
}


pub fn criterion_benchmark(c: &mut Criterion) {
    // initialize market controller
    let base_asset:BrokerAsset = parse_asset("BTC").unwrap();
    let quote_asset:BrokerAsset = parse_asset("USD").unwrap();
    let base_balance: f64 = 1_000_000_000.0;
    let quote_balance: f64 = 1_000_000_000.0;
    let mut account_to_pub_key: Vec<AccountPubKey> = Vec::new();
    let mut account_to_signed_msg: Vec<AccountSignature> = Vec::new();
    let mut market_controller: AccountController<BrokerAsset> = AccountController::new(base_asset, quote_asset);

    // other helpers
    let mut rng: ThreadRng = rand::thread_rng();
    let mut i_account: u64 = 0;
    let n_accounts = 100;
    // generate n_accounts accounts to transact w/ orderbook
    while i_account < n_accounts{
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();
        account_to_pub_key.push(account_pub_key);
        let sig: AccountSignature  = private_key.sign(&TestDiemCrypto(DUMMY_MESSAGE.to_string()));
        account_to_signed_msg.push(sig);
        market_controller.create_account(&account_pub_key, base_balance, quote_balance).unwrap();
        i_account += 1;
    }
    
    c.bench_function("place_orders_engine_plus_account", |b| b.iter(|| 
        place_orders(black_box(1_000), &mut account_to_pub_key, &mut market_controller, &mut rng)));

    c.bench_function("place_orders_engine_plus_account_plus_sig_verify", |b| b.iter(|| 
        place_orders_signed(black_box(1_000), &mut account_to_pub_key, &mut account_to_signed_msg, &mut market_controller, &mut rng)));
    
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
