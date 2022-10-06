import { Grid, useMediaQuery } from '@mui/material'
import { useTheme } from '@mui/material/styles'
import { styled } from '@mui/system'
import AssetBannerWidget from 'components/AssetBannerWidget'
import ChartWidget from 'components/ChartWidget'
import ContractDetailsWidget from 'components/ContractDetailsWidget'
import OrderBookWidget from 'components/OrderBookWidget'
import PortfolioWidget from 'components/PortfolioWidget'
import TradeHistoryWidget from 'components/TradeHistoryWidget'
import TradeSubmitterWidget from 'components/TradeSubmitterWidget'
import UserHealthWidget from 'components/UserHealthWidget'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import {
  useSetAsks,
  useSetBids,
  useSetMarketDetails,
  useSetRecentTrades,
} from 'hooks/useMarketContext'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { useContext, useEffect, useLayoutEffect, useRef, useState } from 'react'
import { Responsive as ResponsiveGridLayout } from 'react-grid-layout'
import { Helmet } from 'react-helmet'
import { useLocation } from 'react-router-dom'

export const StyledGrid = styled(Grid)(({ theme }) => ({
  backgroundColor: theme.palette.background.component,
}))

export const StyledHeader = styled(Grid)(({ theme }) => ({
  // backgroundColor: theme.palette.background.component,
  '&:hover': {
    //backgroundColor: theme.palette.primary.hover,
    cursor: 'pointer',
  },
  minHeight: 20,
}))

// TODO //
// 1.) add support for expand
// 2.) auto-save layouts on changes
// 3.) tune over-lap scenarios
// 4.) tune row height settings & other params
// 5.) stylize default widget container
// 6.) add layouts for other screen sizes

// used to track user dashboard objects -- not currently implemented
const originalItems = [
  'asset-banner',
  'chart',
  'order-book',
  'recent-trades',
  'trade-submitter',
  'portfolio',
]
interface layout {
  i: string
  x: number
  y: number
  w: number
  h: number
}
// initial layout settings
const initialLayouts = {
  md: [
    { i: 'asset-banner', x: 0, y: 0, w: 19, h: 5 }, // yf-> 5
    { i: 'chart', x: 0, y: 5, w: 15, h: 55 }, // yf-> 60
    { i: 'portfolio', x: 0, y: 56, w: 19, h: 44 }, // yf-> 104

    { i: 'order-book', x: 15, y: 5, w: 4, h: 36 }, // yf-> 41
    { i: 'recent-trades', x: 15, y: 36, w: 4, h: 19 }, // yf -> 60

    { i: 'trade-submitter', x: 19, y: 0, w: 5, h: 41, static: true }, // yf -> 41
    { i: 'user-health', x: 19, y: 41, w: 5, h: 19, static: true }, // yf -> 60
    { i: 'contract-details', x: 19, y: 60, w: 5, h: 44, static: true }, // yf -> 104
  ],
  lg: [
    { i: 'asset-banner', x: 0, y: 0, w: 10, h: 5 }, // yf-> 5
    { i: 'chart', x: 0, y: 5, w: 8, h: 75 }, // yf-> 80
    { i: 'portfolio', x: 0, y: 80, w: 10, h: 35 }, // yf-> 115 // why do we need to go to 115 to fill page?

    { i: 'order-book', x: 8, y: 5, w: 2, h: 55 }, // yf-> 60
    { i: 'recent-trades', x: 8, y: 60, w: 2, h: 20 }, // yf-> 80

    { i: 'trade-submitter', x: 10, y: 0, w: 2, h: 60, static: true }, // yf-> 50
    { i: 'user-health', x: 10, y: 60, w: 2, h: 20, static: true }, // yf-> 80
    { i: 'contract-details', x: 10, y: 80, w: 2, h: 35, static: true }, // yf -> 115
  ],
}
type Layouts = { lg: layout }
// fetch layout from user local storage
function getFromLS(key: string) {
  let ls = {}
  if (global.localStorage) {
    try {
      ls = JSON.parse(global.localStorage.getItem('rgl-8')) || {}
    } catch (e) {
      console.log('error e=', e)
    }
  }
  return ls[key]
}

// save layout from user local storage
function saveToLS(key: string, layout) {
  if (global.localStorage) {
    global.localStorage.setItem(
      'rgl-8',
      JSON.stringify({
        [key]: layout,
      }),
    )
  }
}

// helper function for resizes
function getWindowDimensions() {
  const { innerWidth: width, innerHeight: height } = window
  return {
    width,
    height,
  }
}

export default function TradeDashboard() {
  const theme = useTheme()
  const matchUpLg = useMediaQuery(theme.breakpoints.up('lg'))
  // react-grid-layout benefits from padding on large screens
  const dashboardPadding = matchUpLg ? 25 : 0
  // set layouts to track user dashboard settings -- not currently used
  const [item, setItems] = useState(originalItems)
  const [layouts, setLayouts] = useState(getFromLS('layouts') || initialLayouts)
  // recent trade context data
  const setRecentTrades = useSetRecentTrades()
  const setAsks = useSetAsks()
  const setBids = useSetBids()
  const setMarketDetails = useSetMarketDetails()

  const location = useLocation()
  const { pathname } = useLocation()
  const type =
    (pathname.includes('futures/') && 'futures') ||
    (pathname.includes('trade/') && 'spot') ||
    undefined

  // filter incoming market symbol according to path
  const marketSymbol =
    (pathname.includes('trade/') &&
      pathname.split('trade/')[1].replace('/', '')) ||
    (pathname.includes('futures/') &&
      pathname.split('futures/')[1].replace('/', '')) ||
    'BTC-PERP'
  const { data: selectedMarket } = useSelectMarketDataSymbol(marketSymbol, {
    refetchInterval: 1000,
  })
  const { quoteSymbol, baseSymbol } = selectedMarket || {
    quoteSymbol: 'BTC',
    baseSymbol: 'USD',
  }

  const {
    currentMarketState: { recentTrades },
  } = useContext(CurrentMarketContext)

  // helper functions for layout adjustments -- not currently used
  const onLayoutChange = (_: any, allLayouts: Layouts) => {
    setLayouts(allLayouts)
  }
  const onLayoutSave = () => {
    saveToLS('layouts', layouts)
  }
  const onRemoveItem = (itemId: string) => {
    setItems(items.filter((i) => i !== itemId))
  }
  const onExpandItem = (itemId: string) => {
    // TODO - fix below
    /*const newLayouts = Object.assign({}, layouts)
    const newLayout = newLayouts.lg.filter((layout) => layout.i === itemId)[0]
    newLayout.w = 12
    onLayoutSave()
    setLayouts(newLayouts)
    window.location.reload(false)*/
  }
  const onAddItem = (itemId: string) => {
    setItems([...items, itemId])
  }

  const handleResize = () => {
    setWindowDimensions(getWindowDimensions())
  }

  // sizing functions, used to aid components in properly handling resizes
  const [windowDimensions, setWindowDimensions] = useState(
    getWindowDimensions(),
  )
  const [chartDim, setChartDim] = useState({ width: 0, height: 0 })
  const [orderDim, setOrderDim] = useState({ width: 0, height: 0 })
  const [tradeDim, setTradeDim] = useState({ width: 0, height: 0 })

  useEffect(() => {
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  const chartRef = useRef<HTMLDivElement>(null)
  const orderRef = useRef<HTMLDivElement>(null)
  const tradeRef = useRef<HTMLDivElement>(null)

  useLayoutEffect(() => {
    if (
      chartRef?.current?.clientHeight !== chartDim.height ||
      chartRef?.current?.clientWidth !== chartDim.width
    ) {
      setChartDim({
        height: chartRef?.current?.clientHeight || 0,
        width: chartRef?.current?.clientWidth || 0,
      })
    } else if (
      orderRef?.current?.clientHeight !== orderDim.height ||
      (orderRef?.current?.clientWidth !== orderDim.width &&
        orderRef?.current?.clientWidth > 0)
    ) {
      setOrderDim({
        height: orderRef?.current?.clientHeight || 0,
        width: orderRef?.current?.clientWidth || 0,
      })
    } else if (
      tradeRef?.current?.clientHeight !== tradeDim.height ||
      (tradeRef?.current?.clientWidth !== tradeDim.width &&
        tradeRef?.current?.clientWidth > 0)
    ) {
      setTradeDim({
        height: tradeRef?.current?.clientHeight || 0,
        width: tradeRef?.current?.clientWidth || 0,
      })
    }
  })

  const symbol = marketSymbol ? (marketSymbol.split('-') || '')[0] : 'BTC'

  useEffect(() => {
    setRecentTrades([])
    setAsks([], [])
    setBids([], [])
    setMarketDetails(marketSymbol, type, baseSymbol, quoteSymbol)
  }, [location, quoteSymbol, baseSymbol])

  return (
    <Grid sx={{ mt: -7.25 }}>
      <Helmet>
        <title>{`${recentTrades?.[0]?.price || ''} BTC-PERP | TENEX`}</title>
      </Helmet>
      <ResponsiveGridLayout
        class="react-grid-layout"
        layouts={layouts}
        breakpoints={{ lg: 1500, md: 996, sm: 768, xs: 480, xxs: 0 }}
        cols={{ lg: 12, md: 24, sm: 6, xs: 4, xxs: 2 }}
        rowHeight={1}
        // pad width on large screens
        width={windowDimensions.width - dashboardPadding}
        height={windowDimensions.height}
        onLayoutChange={onLayoutChange}
        // allow for dragging, header is defined in widget class
        draggableHandle=".header"
      >
        <Grid key="asset-banner">
          <AssetBannerWidget />
        </Grid>
        <StyledGrid key="chart">
          <Grid
            // sub-grid & sizing necessary to avoid re-render issues
            ref={chartRef}
            style={{ width: '100%', height: '100%' }}
          >
            {chartRef.current && (
              <ChartWidget width={chartDim.width} height={chartDim.height} />
            )}
          </Grid>
        </StyledGrid>
        <StyledGrid key="order-book">
          <Grid
            // sub-grid is necessary to prevent re-render issues
            ref={orderRef}
            style={{ width: '100%', height: '100%' }}
          >
            {orderRef.current && (
              <OrderBookWidget
                width={orderDim.width}
                height={orderDim.height}
              />
            )}
          </Grid>
        </StyledGrid>
        <StyledGrid key="recent-trades">
          <Grid
            // sub-grid is necessary to prevent re-render issues
            ref={tradeRef}
            style={{ width: '100%', height: '100%', overflowY: 'auto' }}
          >
            {tradeRef.current && (
              <TradeHistoryWidget height={tradeDim.height} />
            )}
          </Grid>
        </StyledGrid>
        <StyledGrid key="trade-submitter">
          <TradeSubmitterWidget />
        </StyledGrid>
        <StyledGrid
          key="portfolio"
          style={{ width: '100%', height: '100%', overflowY: 'auto' }}
        >
          <PortfolioWidget />
        </StyledGrid>
        <StyledGrid key="user-health">
          <UserHealthWidget />
        </StyledGrid>
        <StyledGrid key="contract-details">
          <ContractDetailsWidget />
        </StyledGrid>
      </ResponsiveGridLayout>
    </Grid>
  )
}
