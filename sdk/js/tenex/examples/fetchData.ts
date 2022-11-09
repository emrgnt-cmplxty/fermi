// IMPORTS

// INTERNAL
import TenexClient from '../src/tenex/client'
import { transaction, utils } from '../src/tenex'
import { exampleData } from './data'

// EXTERNAL
import assert from 'assert'
import { getPublicKey } from '@noble/ed25519'

// UTILITIES
// TODO move towards sandbox implementation - https://github.com/fermiorg/fermi/issues/186
const DEFAULT_JSONRPC_ADDRESS = 'http://localhost:3006'

async function main() {
  console.log('Building client')
  let client = new TenexClient(DEFAULT_JSONRPC_ADDRESS)
  console.log('Fetching Market Places')
  const marketPlaces = await client.getFuturesMarketPlaces()
  console.log('Market Places: ', marketPlaces)
  // An example response follows below:
  //
  //
  // Market Places:  [
  //   {
  //     quote_asset_id: 1,
  //     supported_base_asset_ids: [ 0 ],
  //     admin: '0x409aa8642d2b75eaa7ebc6fb5413e2abbc30e78bddef59130570d3066b6c3888'
  //   }
  // ]
  //
  //
  //

  // The following code can only execute if a futures market has been deployed
  let marketPlace = marketPlaces[0]
  console.log('Fetching Markets from First Marketplace')
  const markets = await client.getFuturesMarkets(marketPlace.admin)
  console.log('Markets: ', markets)
  // An example response follows below:
  //
  //
  // Markets:  [
  //   {
  //     max_leverage: 20,
  //     base_asset_id: 0,
  //     quote_asset_id: 1,
  //     open_interest: 100,
  //     last_traded_price: 1000,
  //     oracle_price: 1000000
  //   }
  // ]
  //
  //
  //

  console.log('Fetching Market Admin User Data from Marketplace')
  const marketAdminData = await client.getUserMarketplaceInfo(marketPlace.admin, marketPlace.admin)
  console.log('Market Admin Data: ', marketAdminData)
  // An example response follows below:
  //
  //
  // Market Admin Data:  {
  //   user_deposit: 1000000,
  //   user_collateral_req: 5095001,
  //   user_unrealized_pnl: 99900000,
  //   user_market_info: [ { orders: [Array], position: [Object], base_asset_id: 0 } ],
  //   quote_asset_id: 1
  // }
  //
  //
  //

  assert(marketAdminData.user_market_info.length > 0, 'Failed to fetch user with active data')

  console.log('Zooming in on order data: ', marketAdminData.user_market_info[0].orders)
  // An example response follows below:
  //
  //
  // Zooming in on order data:  [
  //   { order_id: 1, side: 1, quantity: 900, price: 1000 },
  //   { order_id: 2, side: 1, quantity: 1000, price: 1000 }
  //
  //
  //
  console.log('Zooming in on position data: ', marketAdminData.user_market_info[0].position)
  // An example response follows below:
  //
  //
  // Zooming in on position data:  { quantity: 100, side: 1, average_price: 1000 }
  //
  //
  //
  console.log('Fetching Order Book Depth')
  const orderBookDepth = await client.getOrderbookDepth(marketPlace.admin, /* baseAssetId */ 0, /* quoteAssetId */ 1, /* depth */ 10)
  console.log('Order Book Depth Data: ', orderBookDepth)
  // An example response follows below:
  //
  //
  // Order Book Depth Data:  { bids: [ { price: 1000, quantity: 1900 } ], asks: [] }
  //
  //
  //
}
main()
