import { PerpMarket } from '@gdexorg/ifm-client'
import useMangoStore from '../stores/useMangoStore'
import MarketBalances from './MarketBalances'
import MarketPosition from './MarketPosition'

const UserMarketInfo = () => {
  const selectedMarket = useMangoStore((s) => s.selectedMarket.current)
  return selectedMarket instanceof PerpMarket ? (
    <MarketPosition />
  ) : (
    <MarketBalances />
  )
}

export default UserMarketInfo
