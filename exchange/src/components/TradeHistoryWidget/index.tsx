import { Divider, Grid, Typography } from '@mui/material'
import { StyledHeader } from 'components/TradeDashboard'
import WidgetCloseIcon from 'components/WidgetCloseIcon'
import { useSetRecentTrades } from 'hooks/useMarketContext'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { useContext, useState } from 'react'
import useWebSocket from 'react-use-websocket'
import { OrderSide, Trade } from 'utils/globals'

const MAX_TOTAL_TRADES = 25
const UPDATE_DELAY_MS = 500

const formatPrice = (arg: number): string => {
  return arg.toLocaleString('en', {
    useGrouping: true,
    minimumFractionDigits: 2,
  })
}

// processing trade messaging
const processTradeMessages = (
  event: { data: string },
  lastTrade: Trade,
  setLastTrade: (trade: Trade) => void,
  recentTrades: Trade[],
  setRecentTrades: (trades: Trade[]) => void,
) => {
  const response = JSON.parse(event.data)
  if (Number(response.p) > 0) {
    const newTrade = {
      price: Number(response.p),
      qty: Number(response.q),
      time: Number(response.T),
      side: response.m ? OrderSide.ASKS : OrderSide.BIDS,
    }
    const timeDelta = newTrade.time - lastTrade?.time
    if (timeDelta && timeDelta > UPDATE_DELAY_MS) {
      newTrade.qty += lastTrade.qty
      newTrade.side = newTrade.qty > 0 ? OrderSide.BIDS : OrderSide.ASKS
      newTrade.qty = Math.abs(newTrade.qty)
      const tradesCopy = [newTrade].concat(
        recentTrades.slice(0, MAX_TOTAL_TRADES - 1),
      )
      setLastTrade({ ...newTrade, qty: 0 })
      setRecentTrades(tradesCopy)
    } else {
      const sign = newTrade.side === OrderSide.BIDS ? 1 : -1
      setLastTrade({ ...lastTrade, qty: sign * newTrade.qty + lastTrade.qty })
    }
  }
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
        <Typography variant="h7">Recent Trades</Typography>
      </Grid>
      <StyledHeader className="header" sx={{ flexGrow: 1 }} />
      <Grid item style={{ width: 20 }}>
        {isActive && <WidgetCloseIcon />}
      </Grid>
    </Grid>
  )
}

// TODO
// 1.) Move trades into dedicated context
interface TradeHistoryWidgetProps {
  height: number
}

const TradeHistoryWidget = ({ height }: TradeHistoryWidgetProps) => {
  const {
    currentMarketState: { baseSymbol, quoteSymbol, recentTrades },
  } = useContext(CurrentMarketContext)
  const BINANCE_SYMBOL = baseSymbol.split('-')[0].toLowerCase()
  const WSS_FEED_URL = `wss://fstream.binance.com/ws/${BINANCE_SYMBOL}usdt@aggTrade`
  const setRecentTrades = useSetRecentTrades()

  const [lastTrade, setLastTrade] = useState<Trade>({
    qty: 0,
    time: 0,
    side: OrderSide.BIDS,
    price: -1,
  })

  // websocket setup
  useWebSocket(WSS_FEED_URL, {
    onOpen: () =>
      console.log(`Live trades connection opened for ${BINANCE_SYMBOL}.`),
    onClose: () => {
      console.log(`Live trades connection closed for ${BINANCE_SYMBOL}.`)
      //setRecentTrades([])
    },
    shouldReconnect: (closeEvent) => true,
    onMessage: (event: WebSocketEventMap['message']) =>
      processTradeMessages(
        event,
        lastTrade,
        setLastTrade,
        recentTrades,
        setRecentTrades,
      ),
  })

  return (
    <Grid sx={{ height: height }}>
      <HeaderContent />
      <Divider />
      <Grid
        style={{ height: '100%' }}
        sx={{ maxHeight: height - 42.5, mb: 1.25, pl: 1, pr: 1 }}
      >
        <Grid container direction="column" sx={{ pt: 0.5 }}>
          <Grid container direction="row">
            <Grid item xs={4} justifyContent="flex-start">
              <Typography
                variant="caption"
                color="textSecondary"
                alignItems="center"
              >
                {/* {`Price (${quoteSymbol})`} */}
                Price
              </Typography>
            </Grid>
            <Grid
              item
              xs={4}
              container
              justifyContent="flex-end"
              alignItems="center"
            >
              <Typography variant="caption" color="textSecondary">
                {/* {`Size (${baseSymbol})`} */}
                Size
              </Typography>
            </Grid>
            <Grid
              item
              xs={4}
              container
              justifyContent="flex-end"
              alignItems="center"
            >
              <Typography variant="caption" color="textSecondary">
                Time
              </Typography>
            </Grid>
          </Grid>
          {recentTrades.length > 0 &&
            recentTrades.slice(0, MAX_TOTAL_TRADES).map((ele: Trade) => {
              const date = new Date(ele.time)
              return (
                <Grid
                  item
                  key={`${ele.price}-${ele.qty}--${Math.floor(
                    Math.random() * 10000,
                  )}`}
                >
                  <Grid container direction="row">
                    <Grid
                      item
                      xs={4}
                      justifyContent="flex-start"
                      alignItems="center"
                    >
                      <Typography
                        variant="caption"
                        color={
                          ele.side === OrderSide.BIDS
                            ? 'tradeColors.bid'
                            : 'tradeColors.ask'
                        }
                      >
                        {formatPrice(ele.price)}
                      </Typography>
                    </Grid>
                    <Grid
                      item
                      xs={4}
                      container
                      justifyContent="flex-end"
                      alignItems="center"
                    >
                      <Typography variant="caption">
                        {String(ele.qty.toFixed(4))}
                      </Typography>
                    </Grid>
                    <Grid
                      item
                      xs={4}
                      container
                      justifyContent="flex-end"
                      alignItems="center"
                    >
                      <Typography variant="caption">{`${
                        (date.getHours() < 10 ? '0' : '') + date.getHours()
                      }:${
                        (date.getMinutes() < 10 ? '0' : '') + date.getMinutes()
                      }:${
                        (date.getSeconds() < 10 ? '0' : '') + date.getSeconds()
                      }`}</Typography>
                    </Grid>
                  </Grid>
                </Grid>
              )
            })}
        </Grid>
      </Grid>
    </Grid>
  )
}

export default TradeHistoryWidget
