import TwitterIcon from '@mui/icons-material/Twitter'
import {
  Avatar,
  Card,
  Grid,
  Hidden,
  Link,
  List,
  ListItem,
  ListItemText,
  Typography,
} from '@mui/material'
import { Button, Paper } from '@mui/material'
import { styled } from '@mui/system'
import { AssetDisplay } from 'components/AssetDisplay'
import ChartWidget from 'components/ChartWidget'
import DepthChart from 'components/ChartWidget/DepthOfMarketChart'
import OrderBookWidget from 'components/OrderBookWidget'
import TradeHistoryWidget from 'components/TradeHistoryWidget'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import {
  useSetAsks,
  useSetBids,
  useSetMarketDetails,
  useSetRecentTrades,
} from 'hooks/useMarketContext'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useEffect } from 'react'
import { formatNumber } from 'utils/formatters'
import { SYMBOL_TO_IMAGE_DICT } from 'utils/tokenData'

import Image from './chart2.jpeg'

const StyledGrid = styled(Grid)(({ theme }) => ({
  backgroundColor: theme.palette.background.component,
}))

const Header = () => {
  return (
    <Grid container direction="row" sx={{ mb: 5 }}>
      <Grid item sx={{ flexGrow: 1 }} />
      <Grid item sx={{ flexGrow: 1 }} xs={9} sm={7}>
        <Grid container justifyContent="center" sx={{ mb: 2 }}>
          <Typography variant="h1" sx={{ mb: 2 }}>
            DeFi without compromise
          </Typography>
        </Grid>
        <Grid container justifyContent="center">
          <Typography variant="h2">
            A decentralized futures protocol built and owned by the trading
            community
          </Typography>
        </Grid>
        <Grid container direction="row">
          <Button
            variant="contained"
            sx={{ mt: 7 }}
            style={{ color: 'background.component', width: 250 }}
            component={Link}
            href={'/app/futures/BTC-PERP'}
          >
            <Typography color="background.component" variant="h6">
              Start Trading
            </Typography>
          </Button>
          <Button
            sx={{ mt: 7, ml: 3 }}
            variant="outlined"
            style={{ color: 'background.component', width: 150 }}
          >
            <Typography variant="h6">Read Docs</Typography>
          </Button>
          <Grid item sx={{ mt: 8, ml: 3 }}>
            <TwitterIcon />
          </Grid>
        </Grid>
      </Grid>
      <Grid item sx={{ flexGrow: 1 }} />
      <Grid item style={{ width: 300, height: 500 }}>
        <Grid item sx={{ ml: 20, mt: -2, mb: 2, height: 0 }}>
          <AssetDisplay
            symbol={'BTC'}
            rightLabel={'BTC-PERP'}
            rightMetaLabel={'Bitcoin Futures'}
            isActive={false}
            isDropDown={false}
          />
        </Grid>
        <Grid item>{<OrderBookWidget width={300} height={500} />}</Grid>
        <Grid
          sx={{ height: 2, width: 300, mt: 1 }}
          style={{ overflowY: 'hidden' }}
        >
          <TradeHistoryWidget height={1} />
        </Grid>
      </Grid>
    </Grid>
  )
}
interface AssetCardProps {
  symbol: string
  price: number
  returns: number
  isVs?: boolean
}

const AssetCard = ({
  symbol,
  price,
  returns,
  isVs = false,
}: AssetCardProps) => {
  return (
    <Card
      style={{
        height: 350,
        width: 275,
      }}
      sx={{
        '&:hover': {
          boxShadow: `#29eecb 0px 1px 10px 0px`,
          borderRadius: `10px 10px 10px 10px`,
        },
      }}
    >
      <Grid
        container
        alignItems="center"
        justifyContent={'center'}
        sx={{ mt: 7 }}
      >
        <Avatar
          sx={{ width: 40, height: 40 }}
          src={SYMBOL_TO_IMAGE_DICT[symbol.split('-')[0]]?.light}
        />
        {isVs && (
          <Typography variant="h4" sx={{ ml: 2, mr: 2 }}>
            {' '}
            vs{' '}
          </Typography>
        )}
        {isVs && (
          <Avatar
            sx={{ width: 40, height: 40 }}
            src={SYMBOL_TO_IMAGE_DICT[symbol.split('-')[2]]?.light}
          />
        )}
      </Grid>
      <Grid container alignItems="center" justifyContent={'center'}>
        <Typography variant="h4" sx={{ mt: 2 }}>
          {symbol}
        </Typography>
      </Grid>
      <Grid container alignItems="center" justifyContent={'center'}>
        <Typography variant="h5" color="textSecondary" sx={{ mt: 0 }}>
          {`$${formatNumber(price, 0)}`}
        </Typography>
      </Grid>

      <Grid container alignItems="center" justifyContent={'center'}>
        <Typography
          variant="h2"
          color={returns > 0 ? 'tradeColors.bid' : 'tradeColors.ask'}
          sx={{ mt: 2 }}
        >
          {`${returns > 0 ? '+' : ''}${(returns * 100).toFixed(2)}%`}
        </Typography>
      </Grid>
      <Grid container sx={{ pl: 2, pr: 2, mt: 6 }}>
        <Button fullWidth variant="contained">
          {' '}
          Trade Now{' '}
        </Button>
      </Grid>
    </Card>
  )
}

const SuperPowersChart = () => {
  return (
    <>
      <Grid
        style={{ float: 1, height: 0, zIndex: 100000, borderRadius: 16 }}
        sx={{ mb: -97.5, ml: 70 }}
      >
        <Typography variant="h3"> Superpowers for DeFi traders</Typography>
        <List sx={{ listStyleType: 'disc' }}>
          <ListItem sx={{ display: 'list-item' }}>
            <ListItemText>
              {' '}
              <Typography variant="h4" color="white">
                {' '}
                Trade with up to 20x leverage
              </Typography>
            </ListItemText>
          </ListItem>
          <ListItem sx={{ display: 'list-item' }}>
            <ListItemText>
              {' '}
              <Typography variant="h4" color="white">
                {' '}
                Hassle-free 3x leveraged tokens
              </Typography>
            </ListItemText>
          </ListItem>
          <ListItem sx={{ display: 'list-item' }}>
            <ListItemText>
              {' '}
              <Typography variant="h4" color="white">
                {' '}
                Yield and enhanced returns through managed vaults
              </Typography>
            </ListItemText>
          </ListItem>
        </List>
      </Grid>
      <Grid
        container
        alignItems="flex-start"
        sx={{ border: '3px solid #2b2b2b', borderRadius: 1 }}
        style={{ width: 1000 }}
      >
        <img src={Image} />
      </Grid>
    </>
  )
}
const Landing = () => {
  const setRecentTrades = useSetRecentTrades()
  const setAsks = useSetAsks()
  const setBids = useSetBids()
  const setMarketDetails = useSetMarketDetails()
  const {
    currentMarketState: { marketSymbol },
  } = useContext(CurrentMarketContext)
  useEffect(() => {
    setRecentTrades([])
    setAsks([], [])
    setBids([], [])
    setMarketDetails('BTC-PERP', 'futures', 'BTC', 'USD')
  }, [])

  const { data: btcMarket } = useSelectMarketDataSymbol('BTC-PERP', {
    refetchInterval: 1000,
  })
  const { data: ethMarket } = useSelectMarketDataSymbol('ETH-PERP', {
    refetchInterval: 1000,
  })

  return (
    <Grid container justifyContent="center">
      <Grid
        container
        sx={{ maxWidth: 1000, ml: 10, mr: 10 }}
        alignItems="center"
      >
        <Header />
        <Grid sx={{ mt: 2, flexGrow: 1 }}>
          <AssetCard
            symbol={'BTC-3L'}
            price={btcMarket?.indexPrice}
            returns={3 * btcMarket?.dailyChange}
          />
        </Grid>
        <Grid sx={{ mt: 2, flexGrow: 1 }}>
          <AssetCard
            symbol={'ETH-PERP'}
            price={ethMarket?.indexPrice}
            returns={ethMarket?.dailyChange}
          />
        </Grid>
        <Grid sx={{ mt: 2, ml: 2 }}>
          <Grid sx={{ mt: 2, flexGrow: 1 }}>
            <AssetCard
              symbol={'ETH-vs-BTC-10L'}
              price={ethMarket?.indexPrice}
              returns={10 * (ethMarket?.dailyChange - btcMarket?.dailyChange)}
              isVs={true}
            />
          </Grid>
        </Grid>
        <Grid container style={{ height: 100 }} />
        <SuperPowersChart />
      </Grid>
    </Grid>
  )
}
export default Landing

// // import reactLogo from 'assets/images/react-logo.svg'
// import { ReactElement, useState } from 'react'
// import { Link } from 'react-router-dom'

// function Landing(): ReactElement {
//   const [count, setCount] = useState(0)

//   return (
//     <div className="App">
//       <header className="App-header">
//         {/* <img src={reactLogo} className="App-logo" alt="logo" /> */}

//         <p>Hello Vite + React!</p>

//         <Link to="/app">Navigate to App</Link>

//         <p>
//           <button type="button" onClick={() => setCount((count) => count + 1)}>
//             count is: {count}
//           </button>
//         </p>

//         <p>
//           Edit <code>App.tsx</code> and save to test HMR updates.
//         </p>

//         <p>
//           <a
//             className="App-link"
//             href="https://reactjs.org"
//             target="_blank"
//             rel="noopener noreferrer"
//           >
//             Learn React
//           </a>
//           {' | '}
//           <a
//             className="App-link"
//             href="https://vitejs.dev/guide/features.html"
//             target="_blank"
//             rel="noopener noreferrer"
//           >
//             Vite Docs
//           </a>
//         </p>
//       </header>
//     </div>
//   )
// }

// export default Landing
