import { createContext, Dispatch, ReactNode, useReducer, useRef } from 'react'
import {
  BaseSymbol,
  MarketSymbol,
  OrderType,
  OrderTypeMap,
} from 'utils/globals'

//anything here will be set via the order related hooks, other data can be derived using selectors below
export type CurrentOrder = {
  marketSymbol: MarketSymbol | undefined
  baseSymbol: BaseSymbol | undefined
  quoteSymbol: BaseSymbol | undefined
  estimatedPrice: number
  orderType: keyof typeof OrderTypeMap
  orderPrice: number | 'MARKET'
  orderSizeBase: number
  orderSizeBaseFormatted: string
  orderSizeQuote: number
  orderSizeQuoteFormatted: string
  reduceOnly: boolean
  postOnly: boolean
  includeStopLoss: boolean
  includeTakeProfit: boolean
  orderMode: 'untilCancel' | 'immediateOrCancel' | 'fillOrKill'
}

export const orderModeToName = {
  untilCancel: 'Good-till-cancel',
  immediateOrCancel: 'Immediate-or-cancel',
  fillOrKill: 'Fill-or-Kill',
}
const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
  return OrderTypeMap[ele as keyof typeof OrderTypeMap] == OrderType.Market
})
const initialState: CurrentOrder = {
  marketSymbol: undefined,
  baseSymbol: undefined,
  quoteSymbol: undefined,
  estimatedPrice: 0,
  orderType: orderTypeMarket,
  orderPrice: 'MARKET',
  orderSizeBase: 0,
  orderSizeBaseFormatted: '',
  orderSizeQuote: 0,
  orderSizeQuoteFormatted: '',
  reduceOnly: false,
  postOnly: false,
  includeStopLoss: false,
  includeTakeProfit: false,
  orderMode: 'untilCancel',
}

export type CurrentOrderAction =
  | {
      type: 'updateCurrentOrder'
      payload: Partial<CurrentOrder>
    }
  | {
      type: 'resetCurrentOrder'
    }

export interface CurrentOrderContextInterface {
  currentOrderState: CurrentOrder
  currentOrderDispatch: Dispatch<CurrentOrderAction>
}

export const CurrentOrderContext = createContext<CurrentOrderContextInterface>({
  currentOrderState: initialState,
  currentOrderDispatch: () => {},
})

interface CurrentOrderProviderProps {
  id: string
  children: ReactNode
}

function CurrentOrderProvider({ children, id }: CurrentOrderProviderProps) {
  const currentOrderReducer = (
    state: CurrentOrder,
    action: CurrentOrderAction,
  ): CurrentOrder => {
    switch (action.type) {
      case 'resetCurrentOrder':
        return initialState
      case 'updateCurrentOrder':
        return Object.assign({}, state, action.payload)
      default:
        return state
    }
  }

  const [currentOrderState, currentOrderDispatch] = useReducer(
    currentOrderReducer,
    initialState as CurrentOrder,
  )

  const currentOrderStateRef = useRef(currentOrderState)
  currentOrderStateRef.current = currentOrderState

  return (
    <CurrentOrderContext.Provider
      value={{
        currentOrderState,
        currentOrderDispatch,
      }}
    >
      {children}
    </CurrentOrderContext.Provider>
  )
}

export default CurrentOrderProvider
