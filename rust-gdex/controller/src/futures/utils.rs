// crate
use super::types::{AccountStateByMarket, CondensedOrder, FuturesMarket, Marketplace, MarketplaceState};
// gdex
use gdex_types::{
    account::AccountPubKey,
    error::GDEXError,
    order_book::OrderSide,
    transaction::{FuturesOrder, FuturesPosition},
};

// HELPER FUNCTIONS
// TODO - move to utils file when controller structure is more mature

// combine two collateral datas into a single data by taking the weighted average of the prices
pub(crate) fn combine_collateral_data(
    old_collateral: &CondensedOrder,
    new_collateral: &CondensedOrder,
) -> CondensedOrder {
    assert!(old_collateral.base_asset_id == new_collateral.base_asset_id);
    assert!(old_collateral.side == new_collateral.side);

    CondensedOrder {
        side: old_collateral.side,
        quantity: old_collateral.quantity + new_collateral.quantity,
        price: (old_collateral.price * old_collateral.quantity + new_collateral.price * new_collateral.quantity)
            / (old_collateral.quantity + new_collateral.quantity),
        base_asset_id: old_collateral.base_asset_id,
    }
}

pub(crate) fn combine_positions(
    mut old_position: FuturesPosition,
    new_position: FuturesPosition,
) -> Option<FuturesPosition> {
    if old_position.side == new_position.side {
        old_position.quantity += new_position.quantity;
        old_position.average_price = (old_position.average_price * old_position.quantity
            + new_position.average_price * new_position.quantity)
            / (old_position.quantity + new_position.quantity);
    } else if old_position.quantity > new_position.quantity {
        old_position.quantity -= new_position.quantity;
    } else if old_position.quantity < new_position.quantity {
        old_position.quantity = new_position.quantity - old_position.quantity;
        old_position.average_price = new_position.average_price;
        old_position.side = new_position.side;
    } else {
        return None;
    }
    Some(old_position)
}

// condenses a stack of orders into a single effective order for collateral calculations
pub(crate) fn condense_orders(open_orders: &[FuturesOrder], base_asset_id: u64) -> (CondensedOrder, CondensedOrder) {
    let mut condensed_bids = CondensedOrder {
        side: OrderSide::Bid as u64,
        quantity: 0,
        base_asset_id,
        price: 0,
    };

    let mut condensed_asks = CondensedOrder {
        side: OrderSide::Ask as u64,
        quantity: 0,
        base_asset_id,
        price: 0,
    };

    for order in open_orders.iter() {
        if order.side == OrderSide::Bid as u64 {
            condensed_bids =
                combine_collateral_data(&CondensedOrder::from_order(order, base_asset_id), &condensed_bids);
        } else {
            condensed_asks =
                combine_collateral_data(&CondensedOrder::from_order(order, base_asset_id), &condensed_asks);
        }
    }
    (condensed_bids, condensed_asks)
}

pub(crate) fn compute_realized_pnl(
    old_position: &FuturesPosition,
    resultant_position: &Option<FuturesPosition>,
    price: u64,
) -> Result<i64, GDEXError> {
    let old_quantity: i64 = old_position.quantity.try_into().map_err(|_| GDEXError::Conversion)?;
    let old_price: i64 = old_position
        .average_price
        .try_into()
        .map_err(|_| GDEXError::Conversion)?;
    let price: i64 = price.try_into().map_err(|_| GDEXError::Conversion)?;
    let price_diff = price - old_price;
    // pnl is positive if price goes up for "long" positions, e.g. those on side of bid
    let multiplier = if old_position.side == OrderSide::Bid as u64 {
        1
    } else {
        -1
    };
    if let Some(resultant_position) = resultant_position {
        let result_quantity: i64 = resultant_position
            .quantity
            .try_into()
            .map_err(|_| GDEXError::Conversion)?;

        if resultant_position.quantity > old_position.quantity {
            Ok(0)
        } else {
            // position has been decreased, realize some profit or loss
            Ok(multiplier * price_diff * (old_quantity - result_quantity))
        }
    } else {
        // position has been closed out, return the entire position value
        Ok(multiplier * old_quantity * price_diff)
    }
}
pub(crate) fn account_market_req_collateral(
    market: &FuturesMarket,
    position: &Option<FuturesPosition>,
    condensed_bids: &CondensedOrder,
    condensed_asks: &CondensedOrder,
) -> u64 {
    let mut market_req_collateral = 0;
    // calculate the worst case collateral by assuming all orders fill in a single direction
    if let Some(position) = position {
        market_req_collateral += position.quantity * market.oracle_price / market.max_leverage;
        // assume worst case of all orders executing 1-sided to calculate collateral req
        if position.side == OrderSide::Bid as u64 {
            market_req_collateral += condensed_bids.price * condensed_bids.quantity / market.max_leverage;
            market_req_collateral += 1;
        } else {
            market_req_collateral += condensed_asks.price * condensed_asks.quantity / market.max_leverage;
            market_req_collateral += 1;
        }
    } else {
        let collateral_consumed = std::cmp::max(
            condensed_asks.price * condensed_asks.quantity,
            condensed_bids.price * condensed_bids.quantity,
        );
        market_req_collateral += collateral_consumed / market.max_leverage + 1;
    }
    market_req_collateral
}

// TODO - don't round up calc when orders and positions are empty
pub(crate) fn get_account_total_req_collateral(
    market_place: &Marketplace,
    account: &AccountPubKey,
    new_order_data: Option<CondensedOrder>,
) -> Result<u64, GDEXError> {
    let mut total_req_collateral = 0;
    // unwrap or fill default with 0 quantity to avoid having to check for existence
    let order_data = new_order_data.unwrap_or(CondensedOrder {
        side: OrderSide::Bid as u64,
        quantity: 0,
        price: 0,
        base_asset_id: 0, // TEMPORARY
    });

    // loop over each market and sum the collateral consumed by the accounts position + orders
    for market in market_place.markets.values() {
        if let Some(account) = market.accounts.get(account) {
            let account_orders = account.open_orders.clone();
            // condense the user orders into a single bid and ask order
            let (mut condensed_bids, mut condensed_asks) = condense_orders(&account_orders, market.base_asset_id);

            // incorporate incoming order if applicable
            if order_data.base_asset_id == market.base_asset_id && order_data.quantity + condensed_bids.quantity > 0 {
                if order_data.side == OrderSide::Bid as u64 {
                    condensed_bids.price = (order_data.price * order_data.quantity
                        + condensed_bids.price * condensed_bids.quantity)
                        / (order_data.quantity + condensed_bids.quantity);
                    condensed_bids.quantity += order_data.quantity;
                } else {
                    condensed_asks.price = (order_data.price * order_data.quantity
                        + condensed_asks.price * condensed_asks.quantity)
                        / (order_data.quantity + condensed_bids.quantity);
                    condensed_asks.quantity += order_data.quantity;
                }
            }
            total_req_collateral +=
                account_market_req_collateral(market, &account.position, &condensed_bids, &condensed_asks);
        } else if order_data.base_asset_id == market.base_asset_id {
            if order_data.side == OrderSide::Bid as u64 {
                total_req_collateral += account_market_req_collateral(
                    market,
                    &None,
                    &order_data,
                    &CondensedOrder {
                        price: 0,
                        quantity: 0,
                        side: OrderSide::Ask as u64,
                        base_asset_id: order_data.base_asset_id,
                    },
                );
            } else {
                total_req_collateral += account_market_req_collateral(
                    market,
                    &None,
                    &CondensedOrder {
                        price: 0,
                        quantity: 0,
                        side: OrderSide::Bid as u64,
                        base_asset_id: order_data.base_asset_id,
                    },
                    &order_data,
                );
            }
        }
    }
    Ok(total_req_collateral)
}

pub(crate) fn get_account_unrealized_pnl(
    market_place: &Marketplace,
    account: &AccountPubKey,
) -> Result<i64, GDEXError> {
    let mut unrealized_pnl = 0;

    // loop over each market and sum the collateral consumed by the accounts position + orders
    for market in market_place.markets.values() {
        if let Some(account) = market.accounts.get(account) {
            if let Some(position) = &account.position {
                // conert the inputs to i64
                let market_latest_price: i64 = market.oracle_price.try_into().map_err(|_| GDEXError::Conversion)?;
                let position_average_price: i64 =
                    position.average_price.try_into().map_err(|_| GDEXError::Conversion)?;
                let position_quantity: i64 = position.quantity.try_into().map_err(|_| GDEXError::Conversion)?;

                // if position side is bid then we are "long" the asset and profit from price increases
                if position.side == OrderSide::Bid as u64 {
                    unrealized_pnl += (market_latest_price - position_average_price) * position_quantity;
                } else {
                    unrealized_pnl += (position_average_price - market_latest_price) * position_quantity;
                }
            }
        }
    }
    Ok(unrealized_pnl)
}

pub(crate) fn get_account_deposit_net_of_req_collateral(
    market_place: &Marketplace,
    account: &AccountPubKey,
) -> Result<i64, GDEXError> {
    let deposit = *market_place
        .deposits
        .lock()
        .unwrap()
        .get(account)
        .ok_or(GDEXError::AccountLookup)?;

    let req_collateral: i64 = get_account_total_req_collateral(market_place, account, None)?
        .try_into()
        .map_err(|_| GDEXError::Conversion)?;
    Ok(deposit - req_collateral)
}

pub(crate) fn get_account_state_by_market(
    market_place: &Marketplace,
    account: &AccountPubKey,
) -> Result<AccountStateByMarket, GDEXError> {
    let mut account_state = Vec::new();
    for market in market_place.markets.values() {
        if let Some(account) = market.accounts.get(account) {
            account_state.push((
                market.base_asset_id,
                account.open_orders.clone(),
                account.position.clone(),
            ));
        }
    }
    Ok(account_state)
}

pub(crate) fn get_marketplace_state(market_place: &Marketplace) -> Result<MarketplaceState, GDEXError> {
    let mut market_state = Vec::new();
    for market in market_place.markets.values() {
        market_state.push(market.clone());
    }
    Ok((market_place.quote_asset_id, market_state))
}
