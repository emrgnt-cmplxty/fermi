// camel casing is used to match the JSON RPC spec
export type MarketplaceResponse = {
  quote_asset_id: number
  supported_base_asset_ids: number[]
  admin: string
}

export type MarketResponse = {
  max_leverage: number
  base_asset_id: number
  quote_asset_id: number
  open_interest: number
  last_traded_price: number
  oracle_price: number
}

export type FuturesPosition = {
  quantity: number
  side: number
  average_price: number
}

export type FuturesOrder = {
  order_id: number
  side: number
  quantity: number
  price: number
}

export type FuturesUserByMarket = {
  orders: FuturesOrder[]
  position: FuturesPosition | undefined
  base_asset_id: number
}

export type MarketplaceUserInfoResponse = {
  user_deposit: number
  user_collateral_req: number
  user_unrealized_pnl: number
  user_market_info: FuturesUserByMarket[]
  quote_asset_id: number
}


export type Depth = {
    price: number,
    quantity: number,
}

export type OrderbookDepthResponse = {
    bids: Depth[],
    asks: Depth[]
}
