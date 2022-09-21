#[path = "./generated/futures_requests.rs"]
#[rustfmt::skip]
#[allow(clippy::all)]
mod futures_requests;
pub use futures_requests::*;

use crate::spot::proto::LimitOrderRequest; // TODO bad, controllers should not depend on eachother like this
impl From<FuturesLimitOrderRequest> for LimitOrderRequest {
    fn from(request: FuturesLimitOrderRequest) -> Self {
        Self {
            base_asset_id: request.base_asset_id,
            quote_asset_id: request.quote_asset_id,
            side: request.side,
            price: request.price,
            quantity: request.quantity,
        }
    }
}

/// Begin externally available testing functions
#[cfg(any(test, feature = "testing"))]
pub mod futures_controller_test_functions {
    use super::*;
    use fastcrypto::DIGEST_LEN;
    use gdex_types::crypto::ToFromBytes;
    use gdex_types::transaction::{create_transaction, serialize_protobuf, ControllerType, RequestType};
    use gdex_types::{account::AccountKeyPair, crypto::KeypairTraits, transaction::SignedTransaction};
    use narwhal_types::CertificateDigest;

    pub const PRIMARY_ASSET_ID: u64 = 0;

    pub fn generate_signed_limit_order(
        kp_sender: &AccountKeyPair,
        kp_admin: &AccountKeyPair,
        base_asset_id: u64,
        quote_asset_id: u64,
        side: u64,
        price: u64,
        quantity: u64,
    ) -> SignedTransaction {
        // TODO replace this with latest

        let request = FuturesLimitOrderRequest {
            base_asset_id,
            quote_asset_id,
            side,
            price,
            quantity,
            market_admin: bytes::Bytes::from(kp_admin.public().as_bytes().to_vec()),
        };

        let dummy_batch_digest = CertificateDigest::new([0; DIGEST_LEN]);

        let fee: u64 = 1000;
        let transaction = create_transaction(
            kp_sender.public().clone(),
            ControllerType::Futures,
            RequestType::FuturesLimitOrder,
            dummy_batch_digest,
            fee,
            serialize_protobuf(&request),
        );

        transaction.sign(kp_sender).unwrap()
    }
}
