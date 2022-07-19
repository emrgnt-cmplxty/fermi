//! 
//! TODO
//! 0.) ADD SIZE CHECKS ON TRANSACTIONS
//! 
extern crate types;

use std::collections::HashMap;

use super::account::{StakeAccount};
use super::bank::{
    BankController,
    STAKE_ASSET_ID,
};
use types::{
    account::{AccountError, AccountPubKey},
};

// The stake controller is responsible for accessing & modifying user balances 
pub struct StakeController
{
    stake_accounts: HashMap<AccountPubKey, StakeAccount>,
}

impl StakeController
{
    pub fn new() -> Self {
        StakeController {
            stake_accounts: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), AccountError> {
        if self.stake_accounts.contains_key(&account_pub_key) {
            Err(AccountError::Creation("Account already exists!".to_string()))
        } else {
            self.stake_accounts.insert(*account_pub_key, StakeAccount::new(*account_pub_key));
            Ok(())
        }
    }

    pub fn get_staked(&self, account_pub_key: &AccountPubKey) -> Result<u64, AccountError> {
        let stake_account: &StakeAccount = self.stake_accounts.get(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        Ok(stake_account.get_staked_amount())
    }

    pub fn stake(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, amount: u64) -> Result<(), AccountError> {
        bank_controller.update_balance(account_pub_key, STAKE_ASSET_ID, -(amount as i64))?;
        let lookup: Option<&mut StakeAccount> = self.stake_accounts.get_mut(account_pub_key);
        match lookup {
            Some(stake_account) => {
                stake_account.set_staked_amount(stake_account.get_staked_amount() + amount as u64);
                Ok(())
            }
            None => {
                let mut new_stake_account: StakeAccount = StakeAccount::new(*account_pub_key);
                new_stake_account.set_staked_amount(amount);
                self.stake_accounts.insert(*account_pub_key, new_stake_account);
                Ok(())
            }
        }
    }

    // TODO #0 //
    pub fn unstake(&mut self, bank_controller: &mut BankController, account_pub_key: &AccountPubKey, amount: u64) -> Result<(), AccountError> {
        bank_controller.update_balance(account_pub_key, STAKE_ASSET_ID, amount as i64)?;
        let stake_account: &mut StakeAccount = self.stake_accounts.get_mut(account_pub_key).ok_or(AccountError::Lookup("Failed to find account".to_string()))?;
        stake_account.set_staked_amount(stake_account.get_staked_amount() - amount);
        Ok(())
    }
}


#[cfg(test)]
mod tests {

    use rand::rngs::{ThreadRng};
    use types::{
        account::{AccountPubKey, AccountPrivKey},
    };
    use diem_crypto::{
        traits::{Uniform},
    };
    
    use super::*;
    use super::super::bank::{CREATED_ASSET_BALANCE, STAKE_ASSET_ID};

    const STAKE_AMOUNT: u64 = 1_000;
    #[test]
    fn stake() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_asset(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key).unwrap();

        let mut stake_controller: StakeController = StakeController::new();
        stake_controller.create_account(&account_pub_key).unwrap();

        stake_controller.stake(&mut bank_controller, &account_pub_key, STAKE_AMOUNT).unwrap();
        assert_eq!(bank_controller.get_balance(&account_pub_key, STAKE_ASSET_ID).unwrap(), CREATED_ASSET_BALANCE - STAKE_AMOUNT);
        assert_eq!(stake_controller.get_staked(&account_pub_key).unwrap(), STAKE_AMOUNT);
    }

    // TODO #0 //
    #[test]
    #[should_panic]
    fn failed_stake() {
        let mut rng: ThreadRng = rand::thread_rng();
        let private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let account_pub_key: AccountPubKey = (&private_key).into();


        let mut bank_controller: BankController = BankController::new();
        bank_controller.create_account(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key).unwrap();
        bank_controller.create_asset(&account_pub_key).unwrap();

        let mut stake_controller: StakeController = StakeController::new();
        stake_controller.create_account(&account_pub_key).unwrap();
        assert_eq!(bank_controller.get_balance(&account_pub_key, STAKE_ASSET_ID).unwrap(), 0);

        stake_controller.stake(&mut bank_controller, &account_pub_key, STAKE_AMOUNT).unwrap();
    }
}