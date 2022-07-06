
use std::fmt::Debug;

#[derive(Debug, Copy, Clone)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct Order<Asset>
where
    Asset: Debug + Clone,
{
    pub order_id: u64,
    pub base_asset: Asset,
    pub quote_asset: Asset,
    pub side: OrderSide,
    pub price: f64,
    pub qty: f64,
}


#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum OrderType {
    Market,
    Limit,
}
