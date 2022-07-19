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
use proc::bank::BankController;
use types::{
    account::{AccountError},
};

pub struct Router {
    manager: ConsensusManager,
}
impl Router {
    pub fn new() -> Self {
        Router {
            manager: ConsensusManager::new(),
        }
    }
    
    pub fn route_transaction(&mut self, txn_request: &TxnRequest<TxnVariant>) -> Result<(), AccountError> {
        // TODO # 1 //
        println!("routing transaction now....");

        txn_request.verify_transaction().unwrap();
        match txn_request.get_txn() {
            &TxnVariant::OrderTransaction(_order) => {
                // DO NOTHING
                return Ok(())
            }
            &TxnVariant::PaymentTransaction(payment) => {
                let bank_controller: &mut BankController = self.manager.get_bank_controller();
                println!("successful match found");
                bank_controller.parse_payment_transaction(&payment)?;
                return Ok(())
            }
            _ => {
                Err(AccountError::OrderProc("Order not matched".to_string()))
            }
        }
    }

    pub fn get_manager(&mut self) -> &mut ConsensusManager {
        &mut self.manager
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

        let mut router: Router = Router::new();
        let consensus_manager = router.get_manager();
        consensus_manager.build_genesis_block().unwrap();
        
        let sender_private_key: &AccountPrivKey = consensus_manager.get_validator_private_key();
        let sender_pub_key: AccountPubKey = consensus_manager.get_validator_pub_key();

        let signed_txn = create_signed_payment_transaction(sender_private_key, sender_pub_key, receiver_pub_key);

        router.route_transaction(&signed_txn).unwrap();
    }
}