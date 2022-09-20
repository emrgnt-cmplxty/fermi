import ArrowDownwardIcon from '@mui/icons-material/ArrowDownward'
import ArrowUpwardIcon from '@mui/icons-material/ArrowUpward'
import { Grid, Typography } from '@mui/material'
import { useSetOrderPrice } from 'hooks/useOrderContext'
import {
  CurrentOrderContext,
  orderModeToName,
} from 'providers/CurrentOrderProvider'
import { useContext, useEffect, useState } from 'react'
import { formatNumber } from 'utils/formatters'
import { OrderType, OrderTypeMap } from 'utils/globals'

enum moveType {
  Positive,
  Negative,
}

interface LatestPriceRowProps {
  latestPrice: number
  markPrice: number
  prevPrice: number
  tickDuration?: number
  suggestedDecimals?: number
}

const LatestPriceRow = ({
  latestPrice,
  markPrice,
  prevPrice,
  tickDuration = 1000,
  suggestedDecimals = 1,
}: LatestPriceRowProps) => {
  const [lastTick, setLastTick] = useState(Math.floor(Date.now()))
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setOrderPrice = useSetOrderPrice()

  useEffect(() => {
    setLastTick(Math.floor(Date.now()))
  }, [latestPrice])

  const timeDelta = Math.floor(Date.now()) - lastTick

  let move = undefined
  if (timeDelta && timeDelta < tickDuration && latestPrice > prevPrice) {
    move = moveType.Positive
  } else if (timeDelta && timeDelta < tickDuration && latestPrice < prevPrice) {
    move = moveType.Negative
  }
  const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Market
  })

  return (
    <Grid container direction="row" sx={{ pt: 1, pb: 0.5 }}>
      <Grid item sx={{ pl: 0.25, pr: 0.25 }}>
        <Typography
          variant="h5"
          style={{ fontWeight: 600 }}
          color={
            (move === moveType.Positive && 'tradeColors.bid') ||
            (move === moveType.Negative && 'tradeColors.ask')
          }
          sx={{ cursor: 'pointer' }}
          onClick={() => {
            if (currentOrderState.orderType !== orderTypeMarket) {
              setOrderPrice(latestPrice)
            }
          }}
        >
          {formatNumber(latestPrice, suggestedDecimals)}
        </Typography>
      </Grid>
      {move === moveType.Positive && (
        <Grid item sx={{ mt: -0.25 }}>
          {' '}
          <ArrowUpwardIcon
            sx={{ color: 'tradeColors.bid', fontSize: 24 }}
          />{' '}
        </Grid>
      )}
      {move === moveType.Negative && (
        <Grid item sx={{ mt: -0.25 }}>
          {' '}
          <ArrowDownwardIcon
            sx={{ color: 'tradeColors.ask', fontSize: 24 }}
          />{' '}
        </Grid>
      )}
      {move !== moveType.Positive && move !== moveType.Negative && (
        <Grid item sx={{ mt: -0.25 }}>
          <ArrowDownwardIcon sx={{ color: 'background.navbar' }} />
        </Grid>
      )}
      <Grid sx={{ flexGrow: 1 }} />
      <Grid item>
        <Typography
          variant="h6"
          color="textSecondary"
          sx={{ cursor: 'pointer' }}
          onClick={() => {
            if (currentOrderState.orderType !== orderTypeMarket) {
              setOrderPrice(
                formatNumber(markPrice, suggestedDecimals).replace(/,/g, ''),
              )
            }
          }}
        >
          {formatNumber(markPrice, suggestedDecimals)}
        </Typography>
      </Grid>
    </Grid>
  )
}

export default LatestPriceRow
