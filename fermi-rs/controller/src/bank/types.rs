// fermi
use fermi_types::{
    account::{AccountBalance, AccountPubKey},
    asset::AssetId,
};
// external
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bank account specifies the format of user accounts in the bank controller
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankAccount {
    account_pub_key: AccountPubKey,
    balances: HashMap<AssetId, AccountBalance>,
}
impl BankAccount {
    pub fn new(account_pub_key: AccountPubKey) -> Self {
        BankAccount {
            account_pub_key,
            balances: HashMap::new(),
        }
    }

    pub fn get_account_pub_key(&self) -> &AccountPubKey {
        &self.account_pub_key
    }

    pub fn get_balances(&self) -> &HashMap<AssetId, AccountBalance> {
        &self.balances
    }

    pub fn get_balance(&self, asset_id: AssetId) -> u64 {
        *self.balances.get(&asset_id).unwrap_or(&0)
    }

    pub fn set_balance(&mut self, asset_id: AssetId, amount: u64) {
        self.balances.insert(asset_id, amount);
    }
}
