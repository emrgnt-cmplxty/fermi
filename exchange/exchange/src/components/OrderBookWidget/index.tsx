import HorizontalSplitIcon from '@mui/icons-material/HorizontalSplit'
import SplitscreenIcon from '@mui/icons-material/Splitscreen'
import VerticalSplitIcon from '@mui/icons-material/VerticalSplit'
import {
  Box,
  Divider,
  FormControl,
  Grid,
  InputLabel,
  MenuItem,
  Select,
  Typography,
} from '@mui/material'
import { StyledHeader } from 'components/TradeDashboard'
import WidgetCloseIcon from 'components/WidgetCloseIcon'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import { useSetAsks, useSetBids } from 'hooks/useMarketContext'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import React, { useContext, useEffect, useState } from 'react'
import useWebSocket from 'react-use-websocket'
import { formatNumber } from 'utils/formatters'
import { roundDown, roundUp } from 'utils/formatters'
import { Order, OrderBookLayout, OrderRung, OrderSide } from 'utils/globals'

import DepthVisualizer from './DepthVisualizer'
import LatestPriceRow from './LatestPriceRow'
import Loader from './Loader'
import PriceLevelRow from './PriceLevelRow'
import TitleRow from './TitleRow'

const TWO_ENTRY_HEIGHT = 190
const ONE_ENTRY_HEIGHT = 170 //175;
const ROW_HEIGHT = 24

const formatPrice = (arg: number): string => {
  return arg.toLocaleString('en', {
    useGrouping: true,
    minimumFractionDigits: 2,
  })
}

const buildPriceLevels = (
  rungs: OrderRung[],
  totalSize: number,
  orderType: OrderSide = OrderSide.BIDS,
  normalizeSize = false,
): React.ReactNode => {
  return rungs.map((rung, idx) => {
    const price: string = formatPrice(rung.price)
    // multiply depth by 2 since we are sampling 5 orders out of a deeper depth
    const depth = (25 * rung.totalSize) / totalSize
    const size: string = normalizeSize
      ? formatNumber(rung.size / rung.price)
      : formatNumber(rung.size)
    const total: string = normalizeSize
      ? formatNumber(rung.totalSize / rung.price)
      : formatNumber(rung.totalSize)

    return (
      <Grid
        key={`${depth}-${size}-${Math.floor(Math.random() * 10000)}`}
        sx={{ pl: 0.25, pr: 0.25 }}
      >
        <PriceLevelRow
          key={size + total}
          total={total}
          size={size}
          price={price}
          orderType={orderType}
        />
        <DepthVisualizer
          key={depth}
          // multipy by 3 to get greater depth
          depth={depth}
          orderType={orderType}
        />
      </Grid>
    )
  })
}
// TODO
// 1.) manually obook depth according to alotted component size
// 2.) Add order book flashing
// 3.) add proper trade data integrations
// 4.) change trade logic to use trade context

const processOrders = (
  newOrders: Order[],
  type: OrderSide,
  rungDelta: number,
  displayDepth: number,
) => {
  let rung: OrderRung = { price: -1, size: 0, totalSize: 0, count: 0 }
  const rungs: OrderRung[] = []
  // process asks
  newOrders.forEach((ele: Order) => {
    const calcPrice =
      type === OrderSide.ASKS
        ? roundUp(ele.price, rungDelta)
        : roundDown(ele.price, rungDelta)
    const calcDelta =
      type === OrderSide.ASKS ? calcPrice - rung.price : rung.price - calcPrice
    rung.totalSize += ele.size
    if (rung.price === -1) {
      rung.price = calcPrice
      rung.size += ele.size
    } else {
      if (calcDelta < rungDelta) {
        rung.size += ele.size
      } else if (rung.count < displayDepth) {
        rungs.push(rung)
        rung = {
          ...ele,
          price: calcPrice,
          totalSize: rung.totalSize,
          count: rung.count + 1,
        }
      }
    }
  })
  if (rungs.length < displayDepth) rungs.push(rung)
  return rungs
}

const HeaderContent = () => {
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
        <Typography variant="h7">Order Book</Typography>
      </Grid>
      <StyledHeader className="header" sx={{ flexGrow: 1 }} />
      <Grid item style={{ width: 20 }}>
        {isActive && <WidgetCloseIcon />}
      </Grid>
    </Grid>
  )
}

interface OrderBookWidgetProps {
  width: number
  height: number
}

const OrderBookWidget = ({ width, height }: OrderBookWidgetProps) => {
  // depth of order book displayed
  const [bookType, setBookType] = useState<
    'bid-and-ask' | 'bid-only' | 'ask-only'
  >('bid-and-ask')
  // modify depth if two-sided
  //const depthModifier = bookType === 'bid-and-ask' ? 2 : 1;
  let oneSideDepth

  if (bookType === 'bid-and-ask') {
    const depthModifier = 2
    oneSideDepth =
      height >= TWO_ENTRY_HEIGHT
        ? parseInt((height - TWO_ENTRY_HEIGHT) / (depthModifier * ROW_HEIGHT)) +
          1
        : 0
  } else {
    const depthModifier = 1
    oneSideDepth =
      height >= ONE_ENTRY_HEIGHT
        ? parseInt((height - ONE_ENTRY_HEIGHT) / (depthModifier * ROW_HEIGHT)) +
          1
        : 0
  }
  const {
    currentMarketState: { bids, asks, recentTrades, marketSymbol },
  } = useContext(CurrentMarketContext)
  const { data: selectedMarket } = useSelectMarketDataSymbol(marketSymbol, {
    refetchInterval: 1000,
  })
  const BINANCE_SYMBOL = selectedMarket?.baseSymbol.split('-')[0].toLowerCase()
  const WSS_FEED_URL = `wss://fstream.binance.com/ws/${BINANCE_SYMBOL}usdt@depth@500ms`
  // const {bids, asks, recentTrades} = currentMarketState;
  // const asks = currentMarketState.asks;
  // const bids = currentMarketState.bids;
  // const recentTrades = currentMarketState.recentTrades;

  const setAsks = useSetAsks()
  const setBids = useSetBids()

  // DUMMY DATA TO TICK ORDER BOOK
  const LAYOUT = OrderBookLayout.VERTICAL

  const [prevPrice, setPrevPrice] = useState(0)
  const [latestPrice, setLatestPrice] = useState(0)
  const [rungDelta, setRungDelta] = useState(selectedMarket?.minPriceIncrement)

  useEffect(() => {
    const recentTrade = recentTrades?.[0]
    if (latestPrice != recentTrade?.price) {
      setPrevPrice(latestPrice)
      setLatestPrice(recentTrade?.price || 0)
    }
  }, [recentTrades])

  // order book data

  // websocket setup
  useWebSocket(WSS_FEED_URL, {
    onOpen: () => console.log(`Live orders connection opened for orderbook`),
    onClose: () => console.log(`Live orders connection closed for orderbook`),
    shouldReconnect: (closeEvent) => true,
    onMessage: (event: WebSocketEventMap['message']) =>
      processOrderMessages(event),
  })

  const processOrderMessages = (event: { data: string }) => {
    // binance websocket depthUpdate payloads
    //
    // "e": "depthUpdate", // Event type
    // "E": 1571889248277, // Event time
    // "T": 1571889248276, // Transaction time
    // "s": "BTCUSDT",
    // "U": 390497796,
    // "u": 390497878,
    // "pu": 390497794,
    // "b": [          // Bids to be updated
    //   [
    //     "7403.89",  // Price Level to be
    //     "0.002"     // Quantity
    //   ], ...

    const response = JSON.parse(event.data)
    const newAsks: Order[] = response.a
      .filter((ele: string[]) => {
        return Number(ele[1]) > 0
      })
      .map((ele: string[]) => {
        return { price: Number(ele[0]), size: Number(ele[1]) }
      }) //.reverse()
    // reverse new bids
    const newBids: Order[] = response.b
      .filter((ele: string[]) => {
        return Number(ele[1]) > 0
      })
      .map((ele: string[]) => {
        return { price: Number(ele[0]), size: Number(ele[1]) }
      })
      .reverse()

    const askRungs = processOrders(
      newAsks,
      OrderSide.ASKS,
      rungDelta,
      oneSideDepth,
    )
    // reverse to match orderbook expectations
    setAsks(askRungs.reverse(), newAsks)

    const bidRungs = processOrders(
      newBids,
      OrderSide.BIDS,
      rungDelta,
      oneSideDepth,
    )
    setBids(bidRungs, newBids)
  }
  const totalSize =
    asks[asks.length - 1]?.totalSize + bids[bids.length - 1]?.totalSize
  return (
    <Grid>
      <HeaderContent />
      <Divider />
      <Grid container sx={{ pr: 1, pl: 1 }}>
        <Grid
          container
          direction="row"
          sx={{ pb: 0.75, pt: 0.75, flexGrow: 1 }}
        >
          <Grid item container xs={6}>
            <Grid item sx={{ flexGrow: 1, cursor: 'pointer' }}>
              <SplitscreenIcon
                onClick={() => {
                  setBookType('bid-and-ask')
                }}
              />
            </Grid>
            <Grid item sx={{ flexGrow: 1, cursor: 'pointer' }}>
              <HorizontalSplitIcon
                sx={{ transform: 'rotate(180deg)' }}
                onClick={() => {
                  setBookType('ask-only')
                }}
              />
            </Grid>
            <Grid item sx={{ flexGrow: 1, cursor: 'pointer' }}>
              <HorizontalSplitIcon
                onClick={() => {
                  setBookType('bid-only')
                }}
              />
            </Grid>
          </Grid>
          <Grid item container xs={6} justifyContent="flex-end">
            {selectedMarket?.minPriceIncrement && (
              <FormControl>
                <InputLabel id="demo-simple-select-label"></InputLabel>
                <Select
                  labelId="demo-simple-select-label"
                  id="demo-simple-select"
                  value={rungDelta}
                  // label="Age"
                  style={{ height: 20, padding: 0, spacing: 0 }}
                  onChange={(event) => {
                    setRungDelta(event?.target?.value)
                  }}
                >
                  <MenuItem value={selectedMarket?.minPriceIncrement}>
                    <Typography variant="caption">
                      {selectedMarket?.minPriceIncrement}
                    </Typography>
                  </MenuItem>
                  <MenuItem value={selectedMarket?.minPriceIncrement * 10}>
                    <Typography variant="caption">
                      {selectedMarket?.minPriceIncrement * 10}
                    </Typography>
                  </MenuItem>
                  <MenuItem value={selectedMarket?.minPriceIncrement * 100}>
                    <Typography variant="caption">
                      {selectedMarket?.minPriceIncrement * 100}
                    </Typography>
                  </MenuItem>
                  <MenuItem value={selectedMarket?.minPriceIncrement * 1000}>
                    <Typography variant="caption">
                      {selectedMarket?.minPriceIncrement * 1000}
                    </Typography>
                  </MenuItem>
                  {/* <MenuItem value={selectedMarket?.minPriceIncrement*10000}>
                  <Typography variant="caption">{selectedMarket?.minPriceIncrement*10000}</Typography>
                </MenuItem> */}
                </Select>
              </FormControl>
            )}
          </Grid>
        </Grid>

        {bids.length && asks.length ? (
          OrderBookLayout.VERTICAL === LAYOUT && (
            <Grid
              container
              direction="column"
              style={{ width: width }}
              sx={{ mb: 0.3 }}
            >
              <Grid item sx={{ mb: -3 }}>
                <TitleRow
                  baseSymbol={selectedMarket?.baseSymbol}
                  quoteSymbol={selectedMarket?.quoteSymbol}
                />
              </Grid>
              <Grid item>
                {(bookType == 'bid-and-ask' || bookType === 'ask-only') &&
                  buildPriceLevels(asks, totalSize, OrderSide.ASKS)}
              </Grid>
              <Divider />
              <LatestPriceRow
                latestPrice={latestPrice}
                prevPrice={prevPrice}
                markPrice={selectedMarket?.markPrice}
                suggestedDecimals={selectedMarket?.suggestedDecimals}
              />
              <Divider />
              <Grid item>
                {(bookType == 'bid-and-ask' || bookType === 'bid-only') &&
                  buildPriceLevels(bids, totalSize, OrderSide.BIDS)}
              </Grid>
            </Grid>
          )
        ) : (
          <Box style={{ width: width }} sx={{ mt: -17.5 }}>
            <Loader />
          </Box>
        )}
      </Grid>
    </Grid>
  )
}

export default OrderBookWidget
