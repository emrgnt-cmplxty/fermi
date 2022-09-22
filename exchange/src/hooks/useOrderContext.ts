import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import {
  CurrentOrder,
  CurrentOrderContext,
} from 'providers/CurrentOrderProvider'
import { useContext } from 'react'
import { numberToMinPrecision, numberWithCommas } from 'utils/formatters'
import {
  BaseSymbol,
  MarketSymbol,
  OrderType,
  OrderTypeMap,
} from 'utils/globals'

// TODO - add floor function to quote converter
export const useSetOrderDetails = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setOrderDetails = (
    marketSymbol: MarketSymbol,
    baseSymbol: BaseSymbol,
    quoteSymbol: BaseSymbol,
    estimatedPrice: number,
  ) => {
    const update: Partial<CurrentOrder> = {
      marketSymbol,
      baseSymbol,
      quoteSymbol,
      estimatedPrice,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setOrderDetails
}

export const useSetOrderType = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setOrderType = (orderType: keyof typeof OrderTypeMap) => {
    const update: Partial<CurrentOrder> = {
      orderType,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setOrderType
}

export const useSetOrderPrice = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setOrderPrice = (orderPrice: number | 'MARKET') => {
    const update: Partial<CurrentOrder> = {
      orderPrice,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setOrderPrice
}

export const useSetOrderSizeBase = () => {
  const { currentOrderState, currentOrderDispatch } =
    useContext(CurrentOrderContext)
  const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Market
  })
  const priceAdjustor =
    currentOrderState.orderType === orderTypeMarket
      ? currentOrderState.estimatedPrice
      : currentOrderState.orderPrice
  const { data: selectedMarket } = useSelectMarketDataSymbol(
    currentOrderState.marketSymbol,
    { refetchInterval: 1000 },
  )
  const { minPriceIncrement, minSizeIncrement, minOrderSize } =
    selectedMarket || {
      minPriceIncrement: 0,
      minSizeIncrement: 0,
      minOrderSize: 0,
    }
  const setOrderSizeBase = (orderSizeBase: string) => {
    const threshOrderSizeBase = numberToMinPrecision(
      orderSizeBase,
      minSizeIncrement,
    )
    const update: Partial<CurrentOrder> = {
      orderSizeBaseFormatted: numberWithCommas(threshOrderSizeBase),
      orderSizeBase: threshOrderSizeBase,
      orderSizeQuote: Number(threshOrderSizeBase) * priceAdjustor,
      orderSizeQuoteFormatted: numberWithCommas(
        Number(
          (threshOrderSizeBase * priceAdjustor).toFixed(minPriceIncrement),
        ),
      ),
    }

    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }

  return setOrderSizeBase
}

export const useSetOrderSizeQuote = () => {
  const { currentOrderState, currentOrderDispatch } =
    useContext(CurrentOrderContext)
  const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Market
  })
  const priceAdjustor = orderTypeMarket
    ? currentOrderState.estimatedPrice
    : currentOrderState.orderPrice
  const { data: selectedMarket } = useSelectMarketDataSymbol(
    currentOrderState.marketSymbol,
    { refetchInterval: 1000 },
  )
  const { minSizeIncrement } = selectedMarket || {
    minPriceIncrement: 0,
    minSizeIncrement: 0,
    minOrderSize: 0,
  }
  const setOrderSizeQuote = (orderSizeQuote: string) => {
    const threshOrderSizeQuote = numberToMinPrecision(orderSizeQuote, 0.01)
    const update: Partial<CurrentOrder> = {
      orderSizeQuote: Number(threshOrderSizeQuote),
      orderSizeQuoteFormatted: numberWithCommas(threshOrderSizeQuote),
      orderSizeBase: Number(threshOrderSizeQuote) / priceAdjustor,
      orderSizeBaseFormatted: numberWithCommas(
        numberToMinPrecision(
          threshOrderSizeQuote / priceAdjustor,
          minSizeIncrement,
        ),
      ),
    }

    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setOrderSizeQuote
}

export const useSetReduceMode = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setReduceMode = (reduceOnly: boolean) => {
    const update: Partial<CurrentOrder> = {
      reduceOnly,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setReduceMode
}

export const useSetPostMode = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setPostMode = (postOnly: boolean) => {
    const update: Partial<CurrentOrder> = {
      postOnly,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setPostMode
}

export const useSetOrderMode = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setOrderMode = (
    orderMode: 'untilCancel' | 'immediateOrCancel' | 'fillOrKill',
  ) => {
    const update: Partial<CurrentOrder> = {
      orderMode,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setOrderMode
}

export const useSetIncludeStopLoss = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setIncludeStopLoss = (includeStopLoss: boolean) => {
    const update: Partial<CurrentOrder> = {
      includeStopLoss,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setIncludeStopLoss
}

export const useSetIncludeTakeProfit = () => {
  const { currentOrderDispatch } = useContext(CurrentOrderContext)

  const setIncludeTakeProfit = (includeTakeProfit: boolean) => {
    const update: Partial<CurrentOrder> = {
      includeTakeProfit,
    }
    currentOrderDispatch({
      type: 'updateCurrentOrder',
      payload: update,
    })
  }
  return setIncludeTakeProfit
}
