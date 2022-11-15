// fermi
use crate::bank::{proto::*, types::BankAccount};
use crate::controller::Controller;
use crate::event_manager::{EventEmitter, EventManager};
use crate::router::ControllerRouter;
use fermi_types::{
    account::AccountPubKey,
    asset::{Asset, AssetId},
    crypto::ToFromBytes,
    error::GDEXError,
    store::RPCStoreHandle,
    transaction::{deserialize_protobuf, Transaction},
};
// mysten
use sui_json_rpc::SuiRpcModule;
// external
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ENUMS

#[derive(Eq, PartialEq)]
pub enum Modifier {
    Increment,
    Decrement,
}

// CONSTANTS

// TODO - https://github.com/fermiorg/fermi/issues/169 - implement coherent system for controller account pubkeys
pub const BANK_CONTROLLER_ACCOUNT_PUBKEY: &[u8] = b"STAKECONTROLLERAAAAAAAAAAAAAAAAA";

// 10 billion w/ 6 decimals, e.g. ALGO creation specs.
pub const CREATED_ASSET_BALANCE: u64 = 10_000_000_000_000_000;

// INTERFACE

/// Bank controller maintains user balances across all initialized assets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankController {
    // controller state
    controller_account: AccountPubKey,
    asset_id_to_asset: HashMap<AssetId, Asset>,
    bank_accounts: HashMap<AccountPubKey, BankAccount>,
    n_assets: u64,
    // shared
    event_manager: Arc<Mutex<EventManager>>,
}

impl Default for BankController {
    fn default() -> Self {
        Self {
            // controller state
            controller_account: AccountPubKey::from_bytes(BANK_CONTROLLER_ACCOUNT_PUBKEY).unwrap(),
            asset_id_to_asset: HashMap::new(),
            bank_accounts: HashMap::new(),
            n_assets: 0,
            // shared state
            event_manager: Arc::new(Mutex::new(EventManager::new())), // TEMPORARY
        }
    }
}

#[async_trait]
impl Controller<crate::bank::rpc_server::JSONRPCService> for BankController {
    fn initialize(&mut self, controller_router: &ControllerRouter) {
        self.event_manager = Arc::clone(&controller_router.event_manager);
    }

    fn initialize_controller_account(&self) -> Result<(), GDEXError> {
        Ok(())
    }

    fn handle_consensus_transaction(&mut self, transaction: &Transaction) -> Result<(), GDEXError> {
        let request_type: BankRequestType = transaction.get_request_type()?;
        match request_type {
            BankRequestType::CreateAsset => {
                let _request: CreateAssetRequest = deserialize_protobuf(&transaction.request_bytes)?;
                let sender = transaction.get_sender()?;
                self.create_asset(&sender)
            }
            BankRequestType::Payment => {
                let request: PaymentRequest = deserialize_protobuf(&transaction.request_bytes)?;
                let sender = transaction.get_sender()?;
                let receiver = request.get_receiver()?;
                self.transfer(&sender, &receiver, request.asset_id, request.quantity)
            }
        }
    }

    fn rpc_is_implemented() -> bool {
        true
    }

    fn generate_json_rpc_module(
        state_manager: Arc<Mutex<ControllerRouter>>,
        rpc_store_handle: Arc<RPCStoreHandle>,
    ) -> Result<jsonrpsee::RpcModule<crate::bank::rpc_server::JSONRPCService>, GDEXError> {
        let result = crate::bank::rpc_server::JSONRPCService::new(state_manager, rpc_store_handle).rpc();
        Ok(result)
    }
}

impl EventEmitter for BankController {
    fn get_event_manager(&mut self) -> &mut Arc<Mutex<EventManager>> {
        &mut self.event_manager
    }
}

impl BankController {
    pub fn check_account_exists(&self, account_pub_key: &AccountPubKey) -> bool {
        self.bank_accounts.contains_key(account_pub_key)
    }

    pub fn create_account(&mut self, account_pub_key: &AccountPubKey) -> Result<(), GDEXError> {
        // do not allow double-creation of a single account
        if self.check_account_exists(account_pub_key) {
            Err(GDEXError::AccountCreation)
        } else {
            self.bank_accounts
                .insert(account_pub_key.clone(), BankAccount::new(account_pub_key.clone()));
            Ok(())
        }
    }

    pub fn get_balance(&self, account_pub_key: &AccountPubKey, asset_id: AssetId) -> Result<u64, GDEXError> {
        let bank_account = self
            .bank_accounts
            .get(account_pub_key)
            .ok_or(GDEXError::AccountLookup)?;

        Ok(bank_account.get_balance(asset_id))
    }

    fn update_balance(
        &mut self,
        account_pub_key: &AccountPubKey,
        asset_id: AssetId,
        quantity: u64,
        increment: Modifier,
    ) -> Result<(), GDEXError> {
        let bank_account = self
            .bank_accounts
            .get_mut(account_pub_key)
            .ok_or(GDEXError::AccountLookup)?;
        let current_balance: u64 = bank_account.get_balance(asset_id);

        // if decrementing balance, check if quantity exceeds existing balance
        if increment == Modifier::Decrement {
            if quantity > current_balance {
                return Err(GDEXError::PaymentRequest);
            };
            bank_account.set_balance(asset_id, current_balance - quantity);
        } else {
            bank_account.set_balance(asset_id, current_balance + quantity);
        }

        Ok(())
    }

    pub fn transfer(
        &mut self,
        sender: &AccountPubKey,
        receiver: &AccountPubKey,
        asset_id: AssetId,
        quantity: u64,
    ) -> Result<(), GDEXError> {
        // return error if insufficient user balance
        let balance = self.get_balance(sender, asset_id)?;
        if balance < quantity {
            return Err(GDEXError::PaymentRequest);
        };

        // if receiver account doesn't exist but asset 0 is being sent, create account
        if !self.check_account_exists(receiver) {
            if asset_id == 0 {
                self.create_account(receiver)?
            } else {
                return Err(GDEXError::AccountLookup);
            }
        };

        self.update_balance(sender, asset_id, quantity, Modifier::Decrement)?;
        self.update_balance(receiver, asset_id, quantity, Modifier::Increment)?;

        // emit event
        self.emit_event(&PaymentSuccessEvent::new(sender, receiver, asset_id, quantity));

        Ok(())
    }

    pub fn create_asset(&mut self, owner_pub_key: &AccountPubKey) -> Result<(), GDEXError> {
        // special handling for genesis
        // an account must be created in this instance
        // since account creation is gated by receipt and balance of primary blockchain asset
        if self.n_assets == 0 && !self.check_account_exists(owner_pub_key) {
            self.create_account(owner_pub_key)?
        }

        // throw error if attempting to create asset prior to account creation
        if !self.check_account_exists(owner_pub_key) {
            return Err(GDEXError::AccountCreation);
        }

        // add asset id -> asset mapping to hashmap
        self.asset_id_to_asset.insert(
            self.n_assets,
            Asset {
                asset_id: self.n_assets,
                owner_pubkey: owner_pub_key.clone(),
            },
        );

        self.update_balance(owner_pub_key, self.n_assets, CREATED_ASSET_BALANCE, Modifier::Increment)?;

        // emit event
        self.emit_event(&AssetCreatedEvent::new(self.n_assets));

        // increment asset counter & return less the increment
        self.n_assets += 1;

        Ok(())
    }

    pub fn get_asset(&mut self, asset_id: AssetId) -> Result<&Asset, GDEXError> {
        self.asset_id_to_asset.get(&asset_id).ok_or(GDEXError::AssetLookup)
    }

    pub fn get_num_assets(&mut self) -> u64 {
        self.n_assets
    }
}

// TESTS

#[cfg(test)]
pub mod spot_tests {
    // crate
    use super::*;

    // mysten
    use fastcrypto::{generate_production_keypair, traits::KeyPair as _};
    use narwhal_crypto::KeyPair;

    #[test]
    fn create_and_check_accounts() {
        let mut bank_controller = BankController::default();
        assert!(
            bank_controller.bank_accounts.is_empty(),
            "Bank accounts hashmap must be empty."
        );

        // create account and check
        let user_kp = generate_production_keypair::<KeyPair>();
        bank_controller.create_account(user_kp.public()).unwrap();
        assert!(
            bank_controller.check_account_exists(user_kp.public()),
            "Bank account must exist."
        );

        // check cannot create account again
        assert!(
            bank_controller.create_account(user_kp.public()).is_err(),
            "Cannot create an account twice."
        );

        // create another account and check
        let user_kp1 = generate_production_keypair::<KeyPair>();
        bank_controller.create_account(user_kp1.public()).unwrap();
        assert!(
            bank_controller.check_account_exists(user_kp1.public()),
            "Bank account must exist."
        );
        // check cannot create account again
        assert!(
            bank_controller.create_account(user_kp1.public()).is_err(),
            "Cannot create an account twice."
        );

        // confirm zero balances
        const TEST_ASSET_ID: u64 = 0;
        assert!(
            bank_controller.get_balance(user_kp.public(), TEST_ASSET_ID).unwrap() == 0,
            "Account balance for asset 0 must be 0."
        );
        assert!(
            bank_controller.get_balance(user_kp1.public(), TEST_ASSET_ID).unwrap() == 0,
            "Account balance for asset 0 must be 0."
        );

        // cannot get balances of account that hasn't been created
        let user_kp2 = generate_production_keypair::<KeyPair>();
        assert!(
            bank_controller.get_balance(user_kp2.public(), TEST_ASSET_ID).is_err(),
            "Cannot get balance for account that hasnt been created."
        );
    }

    #[test]
    fn create_asset_and_transfer() {
        let mut bank_controller = BankController::default();
        let user_kp = generate_production_keypair::<KeyPair>();
        const TEST_ASSET_ID: u64 = 0;

        // check account does not exist
        assert!(
            !bank_controller.check_account_exists(user_kp.public()),
            "Account should not exist."
        );
        // create asset
        bank_controller.create_asset(user_kp.public()).unwrap();
        // check account was created
        assert!(
            bank_controller.check_account_exists(user_kp.public()),
            "Account should exist."
        );
        // check asset was created
        assert!(
            bank_controller.get_asset(TEST_ASSET_ID).unwrap().asset_id == TEST_ASSET_ID,
            "Asset ID must be 0."
        );
        // check user's balance was incremented
        assert!(
            bank_controller.get_balance(user_kp.public(), TEST_ASSET_ID).unwrap() == CREATED_ASSET_BALANCE,
            "User balance must be CREATED_ASSET_BALANCE."
        );
        // check the number of assets is 1
        assert!(bank_controller.get_num_assets() == 1, "Number of assets must be 1.");

        // check account creation does not occur on asset 1
        let user_kp1 = generate_production_keypair::<KeyPair>();
        assert!(
            bank_controller.create_asset(user_kp1.public()).is_err(),
            "Account should not exist."
        );

        // create asset
        bank_controller.create_asset(user_kp.public()).unwrap();
        // check asset was created
        assert!(
            bank_controller.get_asset(TEST_ASSET_ID + 1).unwrap().asset_id == TEST_ASSET_ID + 1,
            "Asset ID must be 1."
        );
        // check user's balance was incremented
        assert!(
            bank_controller
                .get_balance(user_kp.public(), TEST_ASSET_ID + 1)
                .unwrap()
                == CREATED_ASSET_BALANCE,
            "User balance must be CREATED_ASSET_BALANCE."
        );
        // check the number of assets is 1
        assert!(bank_controller.get_num_assets() == 2, "Number of assets must be 2.");
    }

    #[test]
    fn create_bank_catchup_state_default() {
        let bank_controller = BankController::default();
        let catchup_state = bank_controller.get_catchup_state();
        assert!(catchup_state.is_ok());
        let catchup_state = catchup_state.unwrap();
        println!("Catchup state is {} bytes", catchup_state.len());

        match bincode::deserialize(&catchup_state) {
            Ok(BankController {
                asset_id_to_asset,
                bank_accounts,
                n_assets,
                ..
            }) => {
                assert_eq!(n_assets, 0);
                assert_eq!(bank_accounts.keys().len(), 0);
                assert_eq!(asset_id_to_asset.keys().len(), 0);
            }
            Err(_) => panic!("deserializing catchup_state_default failed"),
        }
    }

    #[test]
    fn create_bank_catchup_state_big() {
        // create keypairs initially as it is slow to generate keypairs in non-release mode
        let n_users: usize = 1_000;
        let mut keypairs: Vec<KeyPair> = Vec::new();
        let mut bank_controller = BankController::default();
        for _ in 0..n_users {
            let keypair = generate_production_keypair::<KeyPair>();
            bank_controller.create_account(keypair.public()).unwrap();
            bank_controller.create_asset(keypair.public()).unwrap();
            keypairs.push(keypair);
        }

        for i in 0..n_users {
            let sender_kp = &keypairs[i];
            for j in 0..n_users {
                let receiver_kp = &keypairs[j];
                bank_controller
                    .transfer(sender_kp.public(), receiver_kp.public(), i as u64, 1)
                    .unwrap();
            }
        }

        let catchup_state = bank_controller.get_catchup_state();
        assert!(catchup_state.is_ok());
        let catchup_state = catchup_state.unwrap();
        println!(
            "Catchup state is {} GB for {} creators and {} receivers",
            (catchup_state.len() as f64) / 1e9,
            n_users,
            n_users
        );

        match bincode::deserialize(&catchup_state) {
            Ok(BankController {
                asset_id_to_asset,
                bank_accounts,
                n_assets,
                ..
            }) => {
                assert_eq!(n_assets, n_users as u64);
                assert_eq!(bank_accounts.keys().len(), n_users);
                assert_eq!(asset_id_to_asset.keys().len(), n_users);
            }
            Err(_) => panic!("deserializing catchup_state_default failed"),
        }
    }
}