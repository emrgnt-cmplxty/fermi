import { MarketData, OrderData, PositionData } from 'utils/globals'

const spotPairs: MarketData[] = [
  {
    // identifiers
    baseName: 'Bitcoin',
    baseSymbol: 'BTC',
    quoteName: 'USD Circle',
    quoteSymbol: 'USD',
    name: 'Bitcoin vs. USD Spot',
    symbol: 'BTC-USD',
    type: 'spot',
    // parameters
    minPriceIncrement: 1,
    minSizeIncrement: 0.001,
    minOrderSize: 0.001,
    suggestedDecimals: 2,
    // data
    price: 10000.511,
    dailyChange: 0.051,
    dailyLow: 9000,
    dailyHigh: 11000.123,
    dailyVolume: 100000000,
    quotePrecision: 6,
  },
  {
    // identifiers
    baseName: 'Ethereum',
    baseSymbol: 'ETH',
    quoteName: 'USD Circle',
    quoteSymbol: 'USD',
    name: 'Ethereum vs. USD Spot',
    symbol: 'ETH-USD',
    type: 'spot',
    // parameters
    minPriceIncrement: 1,
    minSizeIncrement: 0.001,
    minOrderSize: 0.001,
    suggestedDecimals: 2,
    // data
    price: 1000,
    dailyChange: -0.051,
    dailyLow: 9000,
    dailyHigh: 11000.123,
    dailyVolume: 100000000,
    quotePrecision: 6,
  },
]

export const getSpotPairs = async () => {
  return spotPairs
}

export const symbolInfo = {
  BTC: {
    name: 'Bitcoin',
    minPriceIncrement: 0.1,
    minSizeIncrement: 0.001,
    minOrderSize: 0.001,
    suggestedDecimals: 1,
    quotePrecision: 8,
  },
  ETH: {
    name: 'Ethereum',
    minPriceIncrement: 0.01,
    minSizeIncrement: 0.001,
    minOrderSize: 0.001,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  BNB: {
    name: 'Binance',
    minPriceIncrement: 0.001,
    minSizeIncrement: 0.01,
    minOrderSize: 0.01,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  SOL: {
    name: 'Solana',
    minPriceIncrement: 0.001,
    minSizeIncrement: 0.01,
    minOrderSize: 0.01,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  ALGO: {
    name: 'Algorand',
    minPriceIncrement: 0.0001,
    minSizeIncrement: 0.1,
    minOrderSize: 0.1,
    suggestedDecimals: 4,
    quotePrecision: 8,
  },
  DOGE: {
    name: 'Dogecoin',
    minPriceIncrement: 0.000001,
    minSizeIncrement: 1,
    minOrderSize: 1,
    suggestedDecimals: 6,
    quotePrecision: 8,
  },
  XRP: {
    name: 'Ripple',
    minPriceIncrement: 0.0001,
    minSizeIncrement: 1,
    minOrderSize: 1,
    suggestedDecimals: 4,
    quotePrecision: 8,
  },
  TRON: {
    name: 'Tron',
    minPriceIncrement: 0.001,
    minSizeIncrement: 0.01,
    minOrderSize: 0.01,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  DOT: {
    name: 'Polkadot',
    minPriceIncrement: 0.001,
    minSizeIncrement: 0.01,
    minOrderSize: 0.01,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  AVAX: {
    name: 'Avalanche',
    minPriceIncrement: 0.001,
    minSizeIncrement: 0.01,
    minOrderSize: 0.01,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  ADA: {
    name: 'Cardano',
    minPriceIncrement: 0.00001,
    minSizeIncrement: 1,
    minOrderSize: 1,
    suggestedDecimals: 4,
    quotePrecision: 8,
  },
  LTC: {
    name: 'Lightcoin',
    minPriceIncrement: 0.01,
    minSizeIncrement: 0.001,
    minOrderSize: 0.001,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
  USD: {
    name: 'USD Coin',
    minPriceIncrement: 0.001,
    minSizeIncrement: 0.01,
    minOrderSize: 0.01,
    suggestedDecimals: 2,
    quotePrecision: 8,
  },
}

// ex binance exchange info payload
//
// baseAsset: "BTC"
// baseAssetPrecision: 8
// contractType: "PERPETUAL"
// deliveryDate: 4133404800000
// filters: (7) [{…}, {…}, {…}, {…}, {…}, {…}, {…}]
// liquidationFee: "0.005000"
// maintMarginPercent: "2.5000"
// marginAsset: "USDT"
// marketTakeBound: "0.05"
// onboardDate: 1569398400000
// orderTypes: (7) ['LIMIT', 'MARKET', 'STOP', 'STOP_MARKET', 'TAKE_PROFIT', 'TAKE_PROFIT_MARKET', 'TRAILING_STOP_MARKET']
// pair: "BTCUSDT"
// pricePrecision: 2
// quantityPrecision: 3
// quoteAsset: "USDT"
// quotePrecision: 8
// requiredMarginPercent: "5.0000"
// settlePlan: 0
// status: "TRADING"
// symbol: "BTCUSDT"
// timeInForce: (4) ['GTC', 'IOC', 'FOK', 'GTX']
// triggerProtect: "0.0500"
// underlyingSubType: ['PoW']
// underlyingType: "COIN"

// ex binance funding payload
//
// estimatedSettlePrice: "29980.15414407"
// indexPrice: "29723.02972237"
// interestRate: "0.00010000"
// lastFundingRate: "-0.00000237"
// markPrice: "29718.20000000"
// nextFundingTime: 1654185600000
// symbol: "BTCUSDT"
// time: 1654176428003

// ex binance ticker payload
//
// closeTime: 1654176433494
// count: 4767497
// firstId: 2312638650
// highPrice: "31888.00"
// lastId: 2317406788
// lastPrice: "29724.10"
// lastQty: "0.008"
// lowPrice: "29300.00"
// openPrice: "31818.90"
// openTime: 1654090020000
// priceChange: "-2094.80"
// priceChangePercent: "-6.584"
// quoteVolume: "17096354364.79"
// symbol: "BTCUSDT"
// volume: "564895.705"
// weightedAvgPrice: "30264.62"

export const getFuturesPairs = async () => {
  // return futuresPairs
  const exchangeInfoURL = 'https://fapi.binance.com/fapi/v1/exchangeInfo'
  const genInfoQuery = await fetch(exchangeInfoURL)
  const markets = JSON.parse(await genInfoQuery.text())
  const procMarkets = []
  if (markets && markets.symbols) {
    for (const market of markets.symbols) {
      if (
        Object.keys(symbolInfo).includes(market.baseAsset) &&
        market.quoteAsset === 'USDT' &&
        market.contractType === 'PERPETUAL'
      ) {
        const premiumIndex = `https://fapi.binance.com/fapi/v1/premiumIndex?symbol=${market.baseAsset}${market.quoteAsset}`
        const fundingQuery = await fetch(premiumIndex)
        const fundingData = JSON.parse(await fundingQuery.text())
        const openInterestURL = `https://fapi.binance.com/fapi/v1/openInterest?symbol=${market.baseAsset}${market.quoteAsset}`
        const openInterestQuery = await fetch(openInterestURL)
        const openInterestData = JSON.parse(await openInterestQuery.text())
        const tickerURL = `https://fapi.binance.com/fapi/v1/ticker/24hr?symbol=${market.baseAsset}${market.quoteAsset}`
        const tickerQuery = await fetch(tickerURL)
        const ticker = JSON.parse(await tickerQuery.text())
        const procMarket = {
          baseSymbol: market.baseAsset,
          baseName: symbolInfo[market.baseAsset].name,
          quoteSymbol: 'USD', //market.quoteAsset,
          quoteName: 'USD Coin', //symbolsToNames[market.quoteAsset],
          symbol: `${market.baseAsset}-PERP`,
          type: 'futures',
          price: Number(ticker.lastPrice),
          dailyChange: Number(ticker.priceChangePercent) / 100,
          dailyLow: Number(ticker.lowPrice),
          dailyHigh: Number(ticker.highPrice),
          dailyVolume: Number(ticker.lastPrice) * Number(ticker.volume),
          fundingRate: Number(fundingData.lastFundingRate || 0),
          markPrice: Number(fundingData.markPrice),
          indexPrice: Number(fundingData.indexPrice),
          nextFundingTime: Number(fundingData.nextFundingTime),
          openInterest: Number(openInterestData.openInterest),
          ...symbolInfo[market.baseAsset],
        }
        procMarkets.push(procMarket)
      }
    }
  }
  return procMarkets
}

export const DUMMY_ORDERS = [
  {
    time: 1654263017000,
    symbol: 'XRP-PERP',
    baseSymbol: 'XRP',
    baseName: 'Ripple',
    side: 'Sell',
    type: 'Limit',
    price: 10,
    size: 0.01,
    filled: 0.001,
    reduce: false,
    post: false,
  },
]

export const DUMMY_ADVANCED_ORDERS = [
  {
    time: 1654263017000,
    symbol: 'XRP-PERP',
    baseSymbol: 'XRP',
    baseName: 'Ripple',
    side: 'Sell',
    type: 'Stop Market',
    price: 'N/A',
    triggerPrice: 0.1,
    trailPrice: 'N/A',
    size: 0.01,
    filled: 0.001,
    reduce: false,
    post: false,
  },
]

export const DUMMY_POSITIONS = [
  {
    marketSymbol: 'XRP-PERP',
    baseSymbol: 'XRP',
    baseName: 'Ripple',
    size: 10,
    notional: 10.1,
    entryPrice: 1.01,
    markPrice: 1.02,
    absMarginUsed: 4,
    percMarginUsed: 0.1,
    marginMode: 'cross', // cross or isolated
    pnlQuote: 1,
    pnlBase: 1,
    estLiqPrice: 0,
    suggestedDecimals: 3,
    takeProfit: 1.15,
    stopLoss: 0.9,
  },
  {
    marketSymbol: 'BTC-PERP',
    baseSymbol: 'BTC',
    baseName: 'Bitcoin',
    size: 1,
    notional: 31034.1,
    entryPrice: 33123.1,
    markPrice: 31034.1,
    absMarginUsed: 0.2,
    percMarginUsed: 73,
    marginMode: 'cross', // cross or isolated
    pnlQuote: -3103,
    pnlBase: -0.1043,
    estLiqPrice: 0,
    suggestedDecimals: 1,
  },
  {
    marketSymbol: 'ETH-PERP',
    baseSymbol: 'ETH',
    baseName: 'Ethereum',
    size: -100,
    notional: 200034.1,
    entryPrice: 1901.1,
    markPrice: 2000.1,
    absMarginUsed: 0.2,
    percMarginUsed: 73,
    marginMode: 'cross', // cross or isolated
    pnlQuote: -100000,
    pnlBase: -50,
    estLiqPrice: 3000,
    suggestedDecimals: 2,
  },
]

export const DUMMY_ORDER_HISTORY = [
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 1000000,
    filled: 1000000,
    reduce: false,
    post: false,
    status: 'Filled',
  },
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 100000,
    filled: 0,
    reduce: false,
    post: false,
    status: 'Canceled',
  },
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 100000,
    filled: 0,
    reduce: false,
    post: false,
    status: 'Canceled',
  },
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 100000,
    filled: 0,
    reduce: false,
    post: false,
    status: 'Canceled',
  },
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 100000,
    filled: 0,
    reduce: false,
    post: false,
    status: 'Canceled',
  },
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 100000,
    filled: 0,
    reduce: false,
    post: false,
    status: 'Canceled',
  },
  {
    time: 1654263017000,
    marketSymbol: 'XRP-PERP',
    side: 'Sell',
    type: 'Limit',
    triggerPrice: 'N/A',
    limitPrice: 1.05,
    avgPrice: 1.05,
    size: 100000,
    filled: 0,
    reduce: false,
    post: false,
    status: 'Canceled',
  },
]

interface UserQueriedData {
  positions: PositionData
  openLimitOrders: OrderData
  openAdvancedOrders: OrderData
  orderHistory: OrderData
}
export const getUserData = async (): Promise<UserQueriedData> => {
  return {
    positions: DUMMY_POSITIONS,
    openLimitOrders: DUMMY_ORDERS,
    openAdvancedOrders: DUMMY_ADVANCED_ORDERS,
    orderHistory: DUMMY_ORDER_HISTORY,
  }
}
