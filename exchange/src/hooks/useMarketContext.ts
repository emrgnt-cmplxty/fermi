import {
  CurrentMarket,
  CurrentMarketContext,
} from 'providers/CurrentMarketProvider'
import { useContext } from 'react'
import {
  BaseSymbol,
  MarketSymbol,
  MarketType,
  OrderRung,
  Trade,
} from 'utils/globals'

export const useSetMarketDetails = () => {
  const { currentMarketDispatch } = useContext(CurrentMarketContext)

  const setMarketDetails = (
    marketSymbol: MarketSymbol,
    type: MarketType,
    baseSymbol: BaseSymbol,
    quoteSymbol: BaseSymbol,
  ) => {
    const update: Partial<CurrentMarket> = {
      marketSymbol,
      type,
      baseSymbol,
      quoteSymbol,
    }
    currentMarketDispatch({
      type: 'updateCurrentMarket',
      payload: update,
    })
  }
  return setMarketDetails
}

export const useSetRecentTrades = () => {
  const { currentMarketDispatch } = useContext(CurrentMarketContext)

  const setRecentTrades = (recentTrades: Trade[]) => {
    const update: Partial<CurrentMarket> = {
      recentTrades,
    }
    currentMarketDispatch({
      type: 'updateCurrentMarket',
      payload: update,
    })
  }
  return setRecentTrades
}

export const useSetAsks = () => {
  const { currentMarketDispatch } = useContext(CurrentMarketContext)

  const setAsks = (asks: OrderRung[], rawAsks: any[]) => {
    const update: Partial<CurrentMarket> = {
      asks,
      rawAsks,
    }
    currentMarketDispatch({
      type: 'updateCurrentMarket',
      payload: update,
    })
  }
  return setAsks
}

export const useSetBids = () => {
  const { currentMarketDispatch } = useContext(CurrentMarketContext)

  const setBids = (bids: OrderRung[], rawBids: any[]) => {
    const update: Partial<CurrentMarket> = {
      bids,
      rawBids,
    }
    currentMarketDispatch({
      type: 'updateCurrentMarket',
      payload: update,
    })
  }
  return setBids
}
