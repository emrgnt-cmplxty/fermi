import { getFuturesPairs, getSpotPairs } from 'api'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext } from 'react'
import { useQuery, UseQueryOptions, UseQueryResult } from 'react-query'
import {
  AggregateMarketData,
  MarketData,
  MarketSymbol,
  MarketType,
} from 'utils/globals'

export const useMarketsData = (
  options?: UseQueryOptions<AggregateMarketData, Error>,
) => {
  return useQuery<any, Error>(
    ['marketsData'],
    async () => {
      const futuresData = await getFuturesPairs()
      const spotData = await getSpotPairs()
      const aggData = (futuresData || []).concat(spotData)
      return {
        futures: futuresData,
        spot: spotData,
        // favorites: favoritesData,
        aggregate: aggData,
      }
    },
    {
      ...options,
      enabled: options?.enabled ?? true,
    },
  )
}

export const useSelectMarketDataType = (
  type: MarketType,
  options?: UseQueryOptions<AggregateMarketData, Error>,
): UseQueryResult<MarketData[], Error> => {
  const { settingsState } = useContext(SettingsContext)
  return useMarketsData({
    ...options,
    select: (marketData) =>
      type != 'favorites'
        ? marketData[type]
        : marketData['aggregate'].filter((ele) =>
            settingsState?.favorites?.includes(ele.symbol),
          ),
  })
}

export const useSelectMarketDataSymbol = (
  symbol: MarketSymbol,
  options?: UseQueryOptions<AggregateMarketData, Error>,
): UseQueryResult<MarketData, Error> => {
  return useMarketsData({
    ...options,
    select: (marketData) =>
      marketData['aggregate'].filter((ele) => ele.symbol === symbol)?.[0],
  })
}
