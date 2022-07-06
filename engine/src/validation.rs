
use std::fmt::Debug;

use super::orders::OrderRequest;


/// Validation errors
const ERR_BAD_base_asset: &str = "bad order asset";
const ERR_BAD_quote_asset: &str = "bad price asset";
const ERR_BAD_PRICE_VALUE: &str = "price must be non-negative";
const ERR_BAD_QUANTITY_VALUE: &str = "quantity must be non-negative";
const ERR_BAD_SEQ_ID: &str = "order ID out of range";


/* Validators */

pub struct OrderRequestValidator<Asset> {
    orderbook_base_asset: Asset,
    orderbook_quote_asset: Asset,
    min_sequence_id: u64,
    max_sequence_id: u64,
}

impl<Asset> OrderRequestValidator<Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub fn new(
        orderbook_base_asset: Asset,
        orderbook_quote_asset: Asset,
        min_sequence_id: u64,
        max_sequence_id: u64,
    ) -> Self {
        OrderRequestValidator {
            orderbook_base_asset,
            orderbook_quote_asset,
            min_sequence_id,
            max_sequence_id,
        }
    }


    pub fn validate(&self, request: &OrderRequest<Asset>) -> Result<(), &str> {
        match *request {
            OrderRequest::NewMarketOrder {
                base_asset,
                quote_asset,
                side: _side,
                qty,
                ts: _ts,
            } => self.validate_market(base_asset, quote_asset, qty),

            OrderRequest::NewLimitOrder {
                base_asset,
                quote_asset,
                side: _side,
                price,
                qty,
                ts: _ts,
            } => self.validate_limit(base_asset, quote_asset, price, qty),

            OrderRequest::AmendOrder {
                id,
                price,
                side: _side,
                qty,
                ts: _ts,
            } => self.validate_amend(id, price, qty),

            OrderRequest::CancelOrder { id, side: _side } => self.validate_cancel(id),
        }
    }

    /* Internal validators */

    fn validate_market(
        &self,
        base_asset: Asset,
        quote_asset: Asset,
        qty: f64,
    ) -> Result<(), &str> {

        if self.orderbook_base_asset != base_asset {
            return Err(ERR_BAD_base_asset);
        }

        if self.orderbook_quote_asset != quote_asset {
            return Err(ERR_BAD_quote_asset);
        }

        if qty <= 0.0 {
            return Err(ERR_BAD_QUANTITY_VALUE);
        }

        Ok(())
    }


    fn validate_limit(
        &self,
        base_asset: Asset,
        quote_asset: Asset,
        price: f64,
        qty: f64,
    ) -> Result<(), &str> {

        if self.orderbook_base_asset != base_asset {
            return Err(ERR_BAD_base_asset);
        }

        if self.orderbook_quote_asset != quote_asset {
            return Err(ERR_BAD_quote_asset);
        }

        if price <= 0.0 {
            return Err(ERR_BAD_PRICE_VALUE);
        }

        if qty <= 0.0 {
            return Err(ERR_BAD_QUANTITY_VALUE);
        }

        Ok(())
    }


    fn validate_amend(&self, id: u64, price: f64, qty: f64) -> Result<(), &str> {
        if self.min_sequence_id > id || self.max_sequence_id < id {
            return Err(ERR_BAD_SEQ_ID);
        }

        if price <= 0.0 {
            return Err(ERR_BAD_PRICE_VALUE);
        }

        if qty <= 0.0 {
            return Err(ERR_BAD_QUANTITY_VALUE);
        }

        Ok(())
    }


    fn validate_cancel(&self, id: u64) -> Result<(), &str> {
        if self.min_sequence_id > id || self.max_sequence_id < id {
            return Err(ERR_BAD_SEQ_ID);
        }

        Ok(())
    }
}
