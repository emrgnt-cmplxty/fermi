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
