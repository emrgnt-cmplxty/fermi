import { createContext, Dispatch, ReactNode, useReducer, useRef } from 'react'
import {
  BaseSymbol,
  MarketSymbol,
  MarketType,
  OrderRungs,
  Trade,
} from 'utils/globals'

//anything here will be set via the order related hooks, other data can be derived using selectors below
export type CurrentMarket = {
  marketSymbol: MarketSymbol
  type: MarketType
  baseSymbol: BaseSymbol
  quoteSymbol: BaseSymbol
  recentTrades: Trade[]
  asks: OrderRungs[]
  rawAsks: any[]
  bids: OrderRungs[]
  rawBids: any[]
}

const initialState: CurrentMarket = {
  marketSymbol: 'BTC-PERP',
  type: 'futures',
  baseSymbol: 'BTC',
  quoteSymbol: 'USD',
  recentTrades: [],
  asks: [],
  rawAsks: [],
  bids: [],
  rawBids: [],
}

export type CurrentMarketAction =
  | {
      type: 'updateCurrentMarket'
      payload: Partial<CurrentMarket>
    }
  | {
      type: 'resetCurrentMarket'
    }

export interface CurrentMarketContextInterface {
  currentMarketState: CurrentMarket
  currentMarketDispatch: Dispatch<CurrentMarketAction>
}

export const CurrentMarketContext =
  createContext<CurrentMarketContextInterface>({
    currentMarketState: initialState,
    currentMarketDispatch: () => {},
  })

interface CurrentMarketProviderProps {
  id: string
  children: ReactNode
}

function CurrentMarketProvider({ children, id }: CurrentMarketProviderProps) {
  const currentMarketReducer = (
    state: CurrentMarket,
    action: CurrentMarketAction,
  ): CurrentMarket => {
    switch (action.type) {
      case 'resetCurrentMarket':
        return initialState
      case 'updateCurrentMarket':
        return Object.assign({}, state, action.payload)
      default:
        return state
    }
  }

  const [currentMarketState, currentMarketDispatch] = useReducer(
    currentMarketReducer,
    initialState as CurrentMarket,
  )

  const currentMarketStateRef = useRef(currentMarketState)
  currentMarketStateRef.current = currentMarketState

  return (
    <CurrentMarketContext.Provider
      value={{
        currentMarketState,
        currentMarketDispatch,
      }}
    >
      {children}
    </CurrentMarketContext.Provider>
  )
}

export default CurrentMarketProvider
