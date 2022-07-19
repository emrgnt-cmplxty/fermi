//! THE ROUTER WILL ROUTE TRANSACTIONS TO APPROPRIATE MODULES FOR EXECUTION
//! TO BE IMPLEMENTED
//! 
//! TODO
//! 0.) FIX PREMATURE UNWRAPS
//! 
use super::toy_consensus::{ConsensusManager};
use core::{
    transaction::{
        TxnRequest, 
        TxnVariant,
    },
};
use proc::{
    bank::{BankController}, 
};
use types::{
    account::{AccountError},
};

pub fn route_transaction(consensus_manager: &mut ConsensusManager, txn_request: &TxnRequest<TxnVariant>) -> Result<(), AccountError> {
    txn_request.verify_transaction().unwrap();
    match txn_request.get_txn() {
        &TxnVariant::OrderTransaction(_order) => {
            // DO NOTHING, THIS NEEDS IMPLEMENTING
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
            let (bank_controller,stake_controller) = consensus_manager.get_all_controllers();
            stake_controller.stake(bank_controller, stake.get_from(), stake.get_amount())?;
            return Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use core::{
        transaction::{
            Payment,
            TxnVariant::PaymentTransaction,
        },
    };
    
    use types::{
        account::{AccountPubKey, AccountPrivKey},
    };
    use diem_crypto::{
        hash::CryptoHash,
        SigningKey,
        Uniform,
    };
    use types::spot::DiemCryptoMessage;
    use super::*;
    
    pub fn create_signed_payment_transaction(
        sender_private_key: &AccountPrivKey, 
        sender_pub_key: AccountPubKey, 
        receiver_pub_key: AccountPubKey, 
    ) -> TxnRequest<TxnVariant> {
        
        let txn: TxnVariant = PaymentTransaction(
            Payment::new(
                sender_pub_key,
                receiver_pub_key,
                0,
                10,
            ),
        );
    
        let txn_hash = txn.hash();
        let signed_hash  = sender_private_key.sign(&DiemCryptoMessage(txn_hash.to_string()));
        let signed_txn: TxnRequest<TxnVariant> = TxnRequest::<TxnVariant>::new(
            txn,
            sender_pub_key, 
            signed_hash 
        );
        signed_txn.verify_transaction().unwrap();
        signed_txn
    }
    
    #[test]
    fn test_router() {
        let mut rng = rand::thread_rng();
        let receiver_private_key: AccountPrivKey = AccountPrivKey::generate(&mut rng);
        let receiver_pub_key: AccountPubKey = (&receiver_private_key).into();

        let mut consensus_manager = ConsensusManager::new();
        consensus_manager.build_genesis_block().unwrap();
        
        let sender_private_key: &AccountPrivKey = consensus_manager.get_validator_private_key();
        let sender_pub_key: AccountPubKey = consensus_manager.get_validator_pub_key();

        let signed_txn = create_signed_payment_transaction(sender_private_key, sender_pub_key, receiver_pub_key);

        route_transaction(&mut consensus_manager, &signed_txn).unwrap();
    }
}