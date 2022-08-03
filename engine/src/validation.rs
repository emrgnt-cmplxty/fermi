use types::{AssetId, OrderRequest};

/// Validation errors
const ERR_BAD_BASE_ASSET: &str = "bad order asset";
const ERR_BAD_QUOTE_ASSET: &str = "bad price asset";
const ERR_BAD_PRICE_VALUE: &str = "price must be non-negative";
const ERR_BAD_QUANTITY_VALUE: &str = "quantity must be non-negative";
const ERR_BAD_SEQ_ID: &str = "order ID out of range";

/* Validators */

pub struct OrderRequestValidator {
    orderbook_base_asset: AssetId,
    orderbook_quote_asset: AssetId,
    min_sequence_id: u64,
    max_sequence_id: u64,
}

impl OrderRequestValidator {
    pub fn new(
        orderbook_base_asset: AssetId,
        orderbook_quote_asset: AssetId,
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

    pub fn validate(&self, request: &OrderRequest) -> Result<(), &str> {
        match *request {
            OrderRequest::Market {
                base_asset,
                quote_asset,
                side: _side,
                quantity,
                ts: _ts,
            } => self.validate_market(base_asset, quote_asset, quantity),

            OrderRequest::Limit {
                base_asset,
                quote_asset,
                side: _side,
                price,
                quantity,
                ts: _ts,
            } => self.validate_limit(base_asset, quote_asset, price, quantity),

            OrderRequest::Amend {
                id,
                price,
                side: _side,
                quantity,
                ts: _ts,
            } => self.validate_amend(id, price, quantity),

            OrderRequest::CancelOrder { id, side: _side } => self.validate_cancel(id),
        }
    }

    /* Internal validators */

    fn validate_market(&self, base_asset: AssetId, quote_asset: AssetId, quantity: u64) -> Result<(), &str> {
        if self.orderbook_base_asset != base_asset {
            return Err(ERR_BAD_BASE_ASSET);
        }

        if self.orderbook_quote_asset != quote_asset {
            return Err(ERR_BAD_QUOTE_ASSET);
        }

        if quantity == 0 {
            return Err(ERR_BAD_QUANTITY_VALUE);
        }

        Ok(())
    }

    fn validate_limit(&self, base_asset: AssetId, quote_asset: AssetId, price: u64, quantity: u64) -> Result<(), &str> {
        if self.orderbook_base_asset != base_asset {
            return Err(ERR_BAD_BASE_ASSET);
        }

        if self.orderbook_quote_asset != quote_asset {
            return Err(ERR_BAD_QUOTE_ASSET);
        }

        if price == 0 {
            return Err(ERR_BAD_PRICE_VALUE);
        }

        if quantity == 0 {
            return Err(ERR_BAD_QUANTITY_VALUE);
        }

        Ok(())
    }

    fn validate_amend(&self, id: u64, price: u64, quantity: u64) -> Result<(), &str> {
        if self.min_sequence_id > id || self.max_sequence_id < id {
            return Err(ERR_BAD_SEQ_ID);
        }

        if price == 0 {
            return Err(ERR_BAD_PRICE_VALUE);
        }

        if quantity == 0 {
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
