import {
  Divider,
  Grid,
  Typography,
} from '@mui/material'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { useContext, useState } from 'react'
import { formatNumber } from 'utils/formatters'

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
  const {
    currentMarketState: { marketSymbol, baseSymbol, recentTrades, type },
  } = useContext(CurrentMarketContext)
  const { data: selectedMarket } = useSelectMarketDataSymbol(marketSymbol, {
    refetchInterval: 1000,
  })

  const mark = selectedMarket?.markPrice
  const index = selectedMarket?.indexPrice
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
