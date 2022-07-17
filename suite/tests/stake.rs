//! TODO
//! 1.) Add tests for acc crypto
//! 2.) Move to pre-determined keys to avoid use of RNG
//! 
extern crate proc;
extern crate types;

use rand::rngs::{ThreadRng};

use diem_crypto::{
    traits::{Uniform},
};
use proc::{
    bank::{BankController, CREATED_ASSET_BALANCE, BANK_ACCOUNT_BYTES, STAKE_ASSET_ID,},
    stake::{StakeController}
};
use types::{
    account::{AccountError, AccountPubKey, AccountPrivKey},
    orderbook::OrderSide
};

#[cfg(test)]
mod tests {
    use super::*;
    const TRANSFER_AMOUNT: u64 = 1_000_000;
    const STAKE_AMOUNT: u64 = 1_000;
    #[test]
    fn stake() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key);
        bank_controller.create_asset(&account_pub_key);

        let mut stake_controller: StakeController = StakeController::new();
        stake_controller.create_account(&account_pub_key).unwrap();

        let bank_pub_key: AccountPubKey = AccountPubKey::from_bytes_unchecked(&BANK_ACCOUNT_BYTES).unwrap();
        bank_controller.transfer(&bank_pub_key, &account_pub_key, STAKE_ASSET_ID, TRANSFER_AMOUNT);

        stake_controller.stake(&mut bank_controller, &account_pub_key, STAKE_AMOUNT as i64);
        assert_eq!(bank_controller.get_balance(&bank_pub_key, STAKE_ASSET_ID), CREATED_ASSET_BALANCE - TRANSFER_AMOUNT);
        assert_eq!(bank_controller.get_balance(&account_pub_key, STAKE_ASSET_ID), TRANSFER_AMOUNT - STAKE_AMOUNT);
    }
}