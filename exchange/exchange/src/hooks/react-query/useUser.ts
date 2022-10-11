import { getUserData } from 'api'
import { useWeb3Context } from 'hooks/useWeb3Context'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext } from 'react'
import { useQuery, UseQueryOptions, UseQueryResult } from 'react-query'
import { UserData } from 'utils/globals'

export const useUserData = (options?: UseQueryOptions<UserData, Error>) => {
  const { settingsState } = useContext(SettingsContext)
  const { publicAddress } = useWeb3Context()

  return useQuery<any, Error>(
    ['userData', publicAddress, settingsState],
    async () => {
      const totalCollateralUSD = 1000000
      const usedCollateralUSD = 100000
      const availableCollateralUSD = totalCollateralUSD - usedCollateralUSD
      const longMaxLeverage = 20
      const shortMaxLeverage = 20

      const { openLimitOrders, openAdvancedOrders, orderHistory, positions } =
        await getUserData()

      return {
        totalCollateralUSD,
        usedCollateralUSD,
        availableCollateralUSD,
        longMaxLeverage,
        shortMaxLeverage,
        openLimitOrders,
        openAdvancedOrders,
        orderHistory,
        positions,
      }
    },
    {
      ...options,
      enabled: options?.enabled ?? true,
    },
  )
}
