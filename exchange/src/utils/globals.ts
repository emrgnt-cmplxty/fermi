export const ChainIdToNetwork: Record<number, string> = {
  1: 'mainnet',
  3: 'ropsten',
  4: 'rinkeby',
  5: 'goerli',
  42: 'kovan',
  100: 'xDAI',
  137: 'polygon',
  80001: 'mumbai',
  43114: 'avalanche',
  43113: 'fuji',
  42161: 'arbitrum_one',
  421611: 'arbitrum_rinkeby',
  250: 'fantom_opera',
  4002: 'fantom_testnet',
  10: 'optimism',
  69: 'optimism_kovan',
  1666600000: 'harmony',
  1666700000: 'harmony_testnet',
}

export enum ChainId {
  mainnet = 1,
  ropsten = 3,
  rinkeby = 4,
  goerli = 5,
  kovan = 42,
  xdai = 100,
  polygon = 137,
  mumbai = 80001,
  avalanche = 43114,
  fuji = 43113, // avalanche test network
  arbitrum_one = 42161,
  arbitrum_rinkeby = 421611,
  fantom = 250,
  fantom_testnet = 4002,
  optimism = 10,
  optimism_kovan = 69,
  harmony = 1666600000,
  harmony_testnet = 1666700000,
}

export type MarketType = 'futures' | 'spot' | 'favorites'

export type MarginType = 'cross' | 'isolated'

export interface Order {
  price: number
  size: number
}

export enum OrderSide {
  BIDS,
  ASKS,
}

export enum OrderType {
  Market,
  Limit,
  StopMarket,
  StopLimit,
  Trailing,
  ProfitMarket,
  ProfitLimit,
}

export const OrderTypeMap = {
  Market: OrderType.Market,
  Limit: OrderType.Limit,
  'Stop Market': OrderType.StopMarket,
  'Stop Limit': OrderType.StopLimit,
  Trailing: OrderType.Trailing,
  'Profit Market': OrderType.ProfitMarket,
  'Profit Limit': OrderType.ProfitLimit,
}

export interface OrderData {
  time: number
  marketSymbol: MarketSymbol
  side: 'Sell' | 'Buy'
  type: keyof typeof OrderTypeMap
  triggerPrice: number | 'N/A'
  limitPrice: number | 'N/A'
  avgPrice: number
  size: number
  filled: number
  reduce: boolean
  post: boolean
  status: 'Filled' | 'Open' | 'Canceled'
}

export enum OrderBookLayout {
  VERTICAL,
}

export interface OrderRung {
  price: number
  size: number
  totalSize: number
  count: number
}

export interface Trade {
  price: number
  qty: number
  time: number
  side: OrderSide
}

export const baseSymbols = [
  'BTC',
  'ETH',
  'BNB',
  'SOL',
  'ALGO',
  'DOGE',
  'XRP',
  'TRON',
  'DOT',
  'AVAX',
  'ADA',
  'LTC',
  'USD',
] as const

export type BaseSymbol = typeof baseSymbols[number]

export const marketSymbols = [
  'BTC-PERP',
  'ETH-PERP',
  'BNB-PERP',
  'SOL-PERP',
  'ALGO-PERP',
  'DOGE-PERP',
  'XRP-PERP',
  'TRON-PERP',
  'DOT-PERP',
  'AVAX-PERP',
  'ADA-PERP',
  'LTC-PERP',
  'USD-PERP',
  'BTC-USD',
  'ETH-USD',
] as const

export type MarketSymbol = typeof marketSymbols[number]

export interface PositionData {
  marketSymbol: MarketSymbol
  baseSymbol: BaseSymbol
  baseName: string
  size: number
  entryPrice: number
  markPrice: number
  absMarginUsed: number
  percMarginUsed: number
  marginMode: MarginType
  pnl: number
  suggestedDecimals: number
  takeProfit?: number
  stopLoss?: number
}
export interface MarketData {
  // market identifiers
  baseName: string
  baseSymbol: BaseSymbol
  quoteName: string
  quoteSymbol: BaseSymbol
  name: string
  symbol: MarketSymbol
  type: MarketType
  // market parameters
  minPriceIncrement: number
  minSizeIncrement: number
  minOrderSize: number
  suggestedDecimals: number
  quotePrecision: number
  // ticker data
  price?: number
  dailyChange?: number
  dailyLow?: number
  dailyHigh?: number
  dailyVolume?: number
  // funding data
  fundingRate?: number
  markPrice?: number
  indexPrice?: number
  nextFundingTime?: number
  // oi data
  openInterest?: number
}

export interface AggregateMarketData {
  futures: MarketData[]
  spot: MarketData[]
  favorites: MarketData[]
  aggregate: MarketData[]
}

export interface UserData {
  positions: PositionData
  openLimitOrders: OrderData
  openAdvancedOrders: OrderData
  orderHistory: OrderData
  totalCollateralUSD: number
  usedCollateralUSD: number
  availableCollateralUSD: number
  positionsBySymbol: { [key: MarketSymbol]: Position }
}
