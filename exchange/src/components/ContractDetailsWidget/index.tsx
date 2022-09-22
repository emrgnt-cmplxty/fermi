import {
  Box,
  Button,
  Divider,
  Grid,
  LinearProgress,
  Typography,
} from '@mui/material'
import { StyledHeader } from 'components/TradeDashboard'
import WidgetCloseIcon from 'components/WidgetCloseIcon'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import { useUserData } from 'hooks/react-query/useUser'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import {
  CurrentOrderContext,
  orderModeToName,
} from 'providers/CurrentOrderProvider'
import { useContext, useState } from 'react'
import { rgbToHex } from 'utils/formatters'
import { formatNumber } from 'utils/formatters'
import { toHHMMSS } from 'utils/formatters'

interface HeaderContentProps {
  marketSymbol: string
}
const HeaderContent = ({ marketSymbol }: HeaderContentProps) => {
  const [isActive, setIsActive] = useState(false)
  return (
    <Grid
      container
      direction="row"
      onMouseEnter={() => {
        setIsActive(true)
      }}
      onMouseLeave={() => {
        setIsActive(false)
      }}
    >
      <Grid item sx={{ pl: 1, pt: 0.75, pb: 0.8 }}>
        <Typography variant="caption">
          {`Contract Details ${marketSymbol}`}
        </Typography>
      </Grid>
    </Grid>
  )
}
interface ContractDetailsWidget {
  marketSymbol: string
}

const ContractDetailsWidget = () => {
  const { data: userData } = useUserData()
  const { currentOrderState } = useContext(CurrentOrderContext)

  const health =
    (100 * userData?.usedCollateralUSD) / userData?.totalCollateralUSD
  const AMT_RED = parseInt(String(20 + 2 * health))
  const AMT_GREEN = parseInt(String(200 - 1.2 * health))
  const AMT_BLUE = parseInt(String(100 - health))
  // green -> red & health = 100
  const startVal = rgbToHex(20, 120, 100)
  const endVal = rgbToHex(AMT_RED, AMT_GREEN, AMT_BLUE)

  const {
    currentMarketState: { marketSymbol, baseSymbol, recentTrades, type },
  } = useContext(CurrentMarketContext)
  const { data: selectedMarket } = useSelectMarketDataSymbol(marketSymbol, {
    refetchInterval: 1000,
  })

  const timeToFunding = selectedMarket?.nextFundingTime - new Date()
  const name =
    selectedMarket?.type === 'futures'
      ? `${selectedMarket?.baseName} Futures`
      : `${selectedMarket?.baseName} Spot`
  const spot = recentTrades?.[0]?.price || selectedMarket?.price
  const mark = selectedMarket?.markPrice
  const index = selectedMarket?.indexPrice
  const delta = selectedMarket?.dailyChange
  const funding = selectedMarket?.fundingRate
  const fundingTimer = toHHMMSS(timeToFunding / 1000)
  const openInterest = selectedMarket?.openInterest
  const high = selectedMarket?.dailyHigh
  const low = selectedMarket?.dailyLow
  const volume = selectedMarket?.dailyVolume

  return (
    <Grid>
      <HeaderContent marketSymbol={marketSymbol} />
      <Divider />
      <Grid container direction="column" sx={{ pr: 2, pl: 2, pt: 1 }}>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              {`Expiration Date:`}
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">{`Perpetual`}</Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              {`Index Price:`}
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">
              {formatNumber(index, selectedMarket?.suggestedDecimals)}
            </Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              {`Mark Price:`}
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">
              {formatNumber(mark, selectedMarket?.suggestedDecimals)}
            </Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              {`Open Interest:`}
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">
              {`${formatNumber(openInterest)} ${selectedMarket?.baseSymbol}`}
            </Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              24h Volume
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">
              {`$${formatNumber(volume, 1)}`}
            </Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              24h High
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">
              {formatNumber(high, selectedMarket?.suggestedDecimals)}
            </Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              24h Low
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">
              {formatNumber(low, selectedMarket?.suggestedDecimals)}
            </Typography>
          </Grid>
        </Grid>
        <Grid container direction="row">
          <Grid item style={{ flexGrow: 1 }}>
            <Typography variant="caption" color="textSecondary">
              {`Contract Size:`}
            </Typography>
          </Grid>
          <Grid item>
            <Typography variant="caption">{`1 ${baseSymbol}`}</Typography>
          </Grid>
        </Grid>
      </Grid>
    </Grid>
  )
}

export default ContractDetailsWidget
