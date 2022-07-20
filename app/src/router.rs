//! 
//! router holds functions responsible for taking incoming transactions 
//! and relaying to appropriate Controllers contained in the ConsensusManager
//! 
//! TODO
//! 0.) fix premature unwraps
//! 
use super::toy_consensus::ConsensusManager;
use core::{
    transaction::{
        CreateAsset,
        CreateOrderBook,
        Order,
        Payment,
        Stake,
        TxnRequest, 
        TxnVariant,
    },
};
use engine::orders::{OrderRequest, new_limit_order_request};
use gdex_crypto::{SigningKey, hash::{CryptoHash, HashValue}};
use proc::bank::BankController;
use std::time::SystemTime;
use types::{
    asset::AssetId,
    account::{AccountPubKey, AccountPrivKey, AccountSignature, AccountError},
    orderbook::OrderSide,
    spot::DiemCryptoMessage,
};

// helper functions for constructing and signing various blockchain transactions
pub fn asset_creation_txn(sender_pub_key: AccountPubKey, sender_private_key: &AccountPrivKey) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::CreateAssetTransaction(CreateAsset{});
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*sender_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
            signed_hash 
        )
    )
}

pub fn orderbook_creation_txn(
    sender_pub_key: AccountPubKey, 
    sender_private_key: &AccountPrivKey, 
    quote_asset_id: AssetId, 
    base_asset_id: AssetId,
) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::CreateOrderbookTransaction(CreateOrderBook::new(quote_asset_id, base_asset_id));
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*sender_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
            signed_hash 
        )
    )
}

pub fn order_transaction(
    sender_pub_key: AccountPubKey, 
    sender_private_key: &AccountPrivKey, 
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    order_side: OrderSide,
    price: u64,
    qty: u64, 
) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    // order construction & submission
    let order: OrderRequest = new_limit_order_request(
        base_asset_id,
        quote_asset_id,
        order_side,
        price,
        qty,
        SystemTime::now()
    );
    
    let txn: TxnVariant = TxnVariant::OrderTransaction(Order::new(order));
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*sender_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
            signed_hash 
        )
    )
}

pub fn stake_txn(validator_pub_key: AccountPubKey, validator_private_key: &AccountPrivKey, amount: u64) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::StakeAssetTransaction(Stake::new(validator_pub_key, amount));
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*validator_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            validator_pub_key, 
            signed_hash 
        )
    )
}

pub fn payment_txn(
    sender_pub_key: AccountPubKey, 
    sender_private_key: &AccountPrivKey, 
    receiver_pub_key: AccountPubKey, 
    asset_id: u64,
    amount: u64
) -> Result<TxnRequest<TxnVariant>, AccountError>  {
    let txn: TxnVariant = TxnVariant::PaymentTransaction(Payment::new(sender_pub_key, receiver_pub_key, asset_id, amount));
    let txn_hash: HashValue = txn.hash();
    let signed_hash: AccountSignature  = (*sender_private_key).sign(&DiemCryptoMessage(txn_hash.to_string()));
    Ok(
        TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
            signed_hash 
        )
    )
}

// take a transaction request and route into appropriate controller function(s)
pub fn route_transaction(consensus_manager: &mut ConsensusManager, txn_request: &TxnRequest<TxnVariant>) -> Result<(), AccountError> {
    // TODO #0 //
    txn_request.verify_transaction().unwrap();
    match txn_request.get_txn() {
        &TxnVariant::OrderTransaction(_order) => {
            let (bank_controller, _stake_controller, spot_controller) = consensus_manager.get_all_controllers();
            spot_controller.parse_limit_order_txn(bank_controller, txn_request)?;
            return Ok(())
        }
        &TxnVariant::PaymentTransaction(payment) => {
            let bank_controller: &mut BankController = consensus_manager.get_bank_controller();
            bank_controller.transfer(payment.get_from(), payment.get_to(), payment.get_asset_id(), payment.get_amount())?;
            return Ok(())
        }
        &TxnVariant::CreateAssetTransaction(_creation) => {
            let bank_controller: &mut BankController = consensus_manager.get_bank_controller();
            bank_controller.create_asset(&txn_request.get_sender())?;
            return Ok(())
        }
        &TxnVariant::StakeAssetTransaction(stake) => {
            let (bank_controller, stake_controller, _spot_controller) = consensus_manager.get_all_controllers();
            stake_controller.stake(bank_controller, stake.get_from(), stake.get_amount())?;
            return Ok(())
        }
        &TxnVariant::CreateOrderbookTransaction(create_orderbook) => {
            let (_bank_controller, _stake_controller, spot_controller) = consensus_manager.get_all_controllers();
            spot_controller.create_orderbook(create_orderbook.get_base_asset_id(), create_orderbook.get_quote_asset_id())?;
            return Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use gdex_crypto::Uniform;
    use proc::bank::CREATED_ASSET_BALANCE;
    use rand::prelude::ThreadRng;
    use types::{
        account::{AccountPubKey, AccountPrivKey},
    };
    
    const STAKE_TRANSACTION_AMOUNT: u64 = 1_000_000;
    #[test]
    fn test_router() {
        let mut rng: ThreadRng = rand::thread_rng();
        let receiver_private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();

        let mut consensus_manager: ConsensusManager = ConsensusManager::new();
        consensus_manager.build_genesis_block().unwrap();
        
        let sender_pub_key: AccountPubKey = consensus_manager.get_validator_pub_key();
        let asset_id: u64 = 0;
        let send_amount: u64 = STAKE_TRANSACTION_AMOUNT+10;
        let signed_txn: TxnRequest<TxnVariant> = payment_txn(sender_pub_key, &consensus_manager.get_validator_private_key(), receiver_pub_key, asset_id, send_amount).unwrap();

        route_transaction(&mut consensus_manager, &signed_txn).unwrap();
        assert!(consensus_manager.get_bank_controller().get_balance(&receiver_pub_key, asset_id).unwrap() == send_amount, "Unexpected balance after making payment");

        let signed_txn: TxnRequest<TxnVariant> = asset_creation_txn(sender_pub_key, &consensus_manager.get_validator_private_key()).unwrap();
        route_transaction(&mut consensus_manager, &signed_txn).unwrap();
        let new_asset_id: u64 = 1;
        assert!(consensus_manager.get_bank_controller().get_balance(&sender_pub_key, new_asset_id).unwrap() == CREATED_ASSET_BALANCE, "Unexpected balance after token creation");

        let signed_txn: TxnRequest<TxnVariant> = stake_txn(receiver_pub_key, &receiver_private_key, STAKE_TRANSACTION_AMOUNT).unwrap();
        route_transaction(&mut consensus_manager, &signed_txn).unwrap();
        assert!(consensus_manager.get_stake_controller().get_staked(&receiver_pub_key).unwrap() == STAKE_TRANSACTION_AMOUNT, "Unexpected balance after staking");

    }
}