//!
//! router holds functions responsible for taking incoming transactions
//! and relaying to appropriate Controllers contained in the ValidatorController
//!
//! TODO
//! 0.) fix premature unwraps
//!
use crate::validator::ValidatorController;
use core::transaction::{
    CreateAssetRequest, CreateOrderbookRequest, OrderRequest, PaymentRequest, StakeRequest, TransactionRequest,
    TransactionVariant,
};
use engine::orders::new_limit_order_request;
use gdex_crypto::{hash::CryptoHash, SigningKey};
use std::time::SystemTime;
use types::{
    account::{AccountPrivKey, AccountPubKey},
    asset::AssetId,
    error::GDEXError,
    orderbook::OrderSide,
    spot::DiemCryptoMessage,
};

// helper functions for constructing and signing various blockchain transactions
pub fn asset_creation_transaction(
    sender_pub_key: AccountPubKey,
    sender_private_key: &AccountPrivKey,
) -> Result<TransactionRequest<TransactionVariant>, GDEXError> {
    let transaction = TransactionVariant::CreateAssetTransaction(CreateAssetRequest {});
    let transaction_hash = transaction.hash();
    let signed_hash = (*sender_private_key).sign(&DiemCryptoMessage(transaction_hash.to_string()));
    Ok(TransactionRequest::<TransactionVariant>::new(
        transaction,
        sender_pub_key,
        signed_hash,
    ))
}

pub fn orderbook_creation_transaction(
    sender_pub_key: AccountPubKey,
    sender_private_key: &AccountPrivKey,
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
) -> Result<TransactionRequest<TransactionVariant>, GDEXError> {
    let transaction =
        TransactionVariant::CreateOrderbookTransaction(CreateOrderbookRequest::new(base_asset_id, quote_asset_id));
    let transaction_hash = transaction.hash();
    let signed_hash = (*sender_private_key).sign(&DiemCryptoMessage(transaction_hash.to_string()));
    Ok(TransactionRequest::<TransactionVariant>::new(
        transaction,
        sender_pub_key,
        signed_hash,
    ))
}

pub fn order_transaction(
    sender_pub_key: AccountPubKey,
    sender_private_key: &AccountPrivKey,
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    order_side: OrderSide,
    price: u64,
    quantity: u64,
) -> Result<TransactionRequest<TransactionVariant>, GDEXError> {
    // order construction & submission
    let order: OrderRequest = new_limit_order_request(
        base_asset_id,
        quote_asset_id,
        order_side,
        price,
        quantity,
        SystemTime::now(),
    );

    let transaction = TransactionVariant::OrderTransaction(order);
    let transaction_hash = transaction.hash();
    let signed_hash = (*sender_private_key).sign(&DiemCryptoMessage(transaction_hash.to_string()));
    Ok(TransactionRequest::<TransactionVariant>::new(
        transaction,
        sender_pub_key,
        signed_hash,
    ))
}

pub fn stake_transaction(
    pub_key: AccountPubKey,
    validator_private_key: &AccountPrivKey,
    amount: u64,
) -> Result<TransactionRequest<TransactionVariant>, GDEXError> {
    let transaction = TransactionVariant::StakeAsset(StakeRequest::new(pub_key, amount));
    let transaction_hash = transaction.hash();
    let signed_hash = (*validator_private_key).sign(&DiemCryptoMessage(transaction_hash.to_string()));
    Ok(TransactionRequest::<TransactionVariant>::new(
        transaction,
        pub_key,
        signed_hash,
    ))
}

pub fn payment_transaction(
    sender_pub_key: AccountPubKey,
    sender_private_key: &AccountPrivKey,
    receiver_pub_key: AccountPubKey,
    asset_id: u64,
    amount: u64,
) -> Result<TransactionRequest<TransactionVariant>, GDEXError> {
    let transaction =
        TransactionVariant::PaymentTransaction(PaymentRequest::new(sender_pub_key, receiver_pub_key, asset_id, amount));
    let transaction_hash = transaction.hash();
    let signed_hash = (*sender_private_key).sign(&DiemCryptoMessage(transaction_hash.to_string()));
    Ok(TransactionRequest::<TransactionVariant>::new(
        transaction,
        sender_pub_key,
        signed_hash,
    ))
}

// take a transaction request and route into appropriate controller function(s)
pub fn route_transaction(
    consensus_manager: &mut ValidatorController,
    transaction_request: &TransactionRequest<TransactionVariant>,
) -> Result<(), GDEXError> {
    // TODO #0 //
    transaction_request.verify_transaction().unwrap();
    execute_transaction(consensus_manager, transaction_request)
}

// take a vector of transaction requests and batch verify before routing appropriate controller function(s)
#[cfg(feature = "batch")]
pub mod batch_functions {
    use super::*;
    use core::transaction::batch_functions::{verify_transaction_batch, verify_transaction_batch_multithreaded};

    pub fn route_transaction_batch(
        consensus_manager: &mut ValidatorController,
        transaction_requests: &[TransactionRequest<TransactionVariant>],
    ) -> Result<(), GDEXError> {
        verify_transaction_batch(transaction_requests).unwrap();
        for order_transaction in transaction_requests.iter() {
            execute_transaction(consensus_manager, order_transaction)?;
        }
        Ok(())
    }

    pub fn route_transaction_batch_multithreaded(
        consensus_manager: &mut ValidatorController,
        transaction_requests: &[TransactionRequest<TransactionVariant>],
        n_threads: u64,
    ) -> Result<(), GDEXError> {
        verify_transaction_batch_multithreaded(transaction_requests.to_vec(), n_threads).unwrap();
        for order_transaction in transaction_requests.iter() {
            execute_transaction(consensus_manager, order_transaction)?;
        }
        Ok(())
    }
}

fn execute_transaction(
    consensus_manager: &mut ValidatorController,
    transaction_request: &TransactionRequest<TransactionVariant>,
) -> Result<(), GDEXError> {
    match transaction_request.get_transaction() {
        TransactionVariant::OrderTransaction(_order) => {
            let (bank_controller, _stake_controller, spot_controller) = consensus_manager.get_all_controllers();
            spot_controller.parse_limit_order_transaction(bank_controller, transaction_request)?;
        }
        TransactionVariant::PaymentTransaction(payment) => {
            let bank_controller = consensus_manager.get_bank_controller();
            bank_controller.transfer(
                payment.get_from(),
                payment.get_to(),
                payment.get_asset_id(),
                payment.get_amount(),
            )?;
        }
        TransactionVariant::CreateAssetTransaction(_creation) => {
            let bank_controller = consensus_manager.get_bank_controller();
            bank_controller.create_asset(transaction_request.get_sender())?;
        }
        TransactionVariant::StakeAsset(stake) => {
            let (bank_controller, stake_controller, _spot_controller) = consensus_manager.get_all_controllers();
            stake_controller.stake(bank_controller, stake.get_from(), stake.get_amount())?;
        }
        TransactionVariant::CreateOrderbookTransaction(create_orderbook) => {
            let (_bank_controller, _stake_controller, spot_controller) = consensus_manager.get_all_controllers();
            spot_controller.create_orderbook(
                create_orderbook.get_base_asset_id(),
                create_orderbook.get_quote_asset_id(),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc::{account::generate_key_pair, bank::CREATED_ASSET_BALANCE};

    const STAKE_TRANSACTION_AMOUNT: u64 = 1_000_000;
    #[test]
    fn test_router() {
        let (receiver_pub_key, receiver_private_key) = generate_key_pair();

        let mut consensus_manager = ValidatorController::new();
        consensus_manager.build_genesis_block().unwrap();

        let sender_pub_key = consensus_manager.get_pub_key();
        let asset_id = 0;
        let send_amount = STAKE_TRANSACTION_AMOUNT + 10;
        let signed_transaction = payment_transaction(
            sender_pub_key,
            consensus_manager.get_private_key(),
            receiver_pub_key,
            asset_id,
            send_amount,
        )
        .unwrap();

        route_transaction(&mut consensus_manager, &signed_transaction).unwrap();
        assert!(
            consensus_manager
                .get_bank_controller()
                .get_balance(&receiver_pub_key, asset_id)
                .unwrap()
                == send_amount,
            "Unexpected balance after making payment"
        );

        let signed_transaction =
            asset_creation_transaction(sender_pub_key, consensus_manager.get_private_key()).unwrap();
        route_transaction(&mut consensus_manager, &signed_transaction).unwrap();
        let new_asset_id = 1;
        assert!(
            consensus_manager
                .get_bank_controller()
                .get_balance(&sender_pub_key, new_asset_id)
                .unwrap()
                == CREATED_ASSET_BALANCE,
            "Unexpected balance after token creation"
        );

        let signed_transaction =
            stake_transaction(receiver_pub_key, &receiver_private_key, STAKE_TRANSACTION_AMOUNT).unwrap();
        route_transaction(&mut consensus_manager, &signed_transaction).unwrap();
        assert!(
            consensus_manager
                .get_stake_controller()
                .get_staked(&receiver_pub_key)
                .unwrap()
                == STAKE_TRANSACTION_AMOUNT,
            "Unexpected balance after staking"
        );
    }
}
