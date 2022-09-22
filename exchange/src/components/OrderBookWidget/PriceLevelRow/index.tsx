import { Grid, Typography } from '@mui/material'
import { useSetOrderPrice } from 'hooks/useOrderContext'
import {
  CurrentOrderContext,
  orderModeToName,
} from 'providers/CurrentOrderProvider'
import { useContext, useEffect, useState } from 'react'
import { OrderSide } from 'utils/globals'
import { OrderType, OrderTypeMap } from 'utils/globals'

interface PriceLevelRowProps {
  total: string
  size: string
  price: string
  orderType: OrderSide
}
const PriceLevelRow = ({
  total,
  size,
  price,
  orderType,
}: PriceLevelRowProps) => {
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setOrderPrice = useSetOrderPrice()
  const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Market
  })

  return (
    <Grid container direction="row">
      <Grid
        item
        xs={4}
        justifyContent="flex-start"
        alignItems="center"
        color={
          orderType === OrderSide.ASKS ? 'tradeColors.ask' : 'tradeColors.bid'
        }
        style={{ zIndex: 1 }}
      >
        <Typography
          variant="orderBook"
          sx={{ pt: -0.25, mb: 0.25, cursor: 'pointer' }}
          onClick={() => {
            if (currentOrderState.orderType !== orderTypeMarket) {
              setOrderPrice(price.replace(/,/g, ''))
            }
          }}
        >
          {price}
        </Typography>
      </Grid>
      <Grid
        item
        xs={4}
        container
        justifyContent="flex-end"
        alignItems="center"
        style={{ zIndex: 1 }}
      >
        <Typography variant="caption" style={{ fontWeight: 'medium' }}>
          {size}
        </Typography>
      </Grid>
      <Grid
        item
        xs={4}
        container
        justifyContent="flex-end"
        alignItems="center"
        style={{ zIndex: 1 }}
      >
        <Typography variant="caption" style={{ fontWeight: 'medium' }}>
          {total}
        </Typography>
      </Grid>
    </Grid>
  )
}
export default PriceLevelRow
