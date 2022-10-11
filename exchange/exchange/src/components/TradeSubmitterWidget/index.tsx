import ArrowDropDownIcon from '@mui/icons-material/ArrowDropDown'
import {
  Button,
  ButtonGroup,
  Checkbox,
  FormControl,
  FormControlLabel,
  Grid,
  InputAdornment,
  InputLabel,
  List,
  ListItemButton,
  MenuItem,
  Select,
  Switch,
  Tab,
  Tabs,
  TextField,
  Typography,
} from '@mui/material'
import LoginModal from 'components/ConnectModal'
import { useUserData } from 'hooks/react-query/useUser'
import {
  useSetIncludeStopLoss,
  useSetIncludeTakeProfit,
  useSetOrderDetails,
  useSetOrderMode,
  useSetOrderPrice,
  useSetOrderSizeBase,
  useSetOrderSizeQuote,
  useSetOrderType,
  useSetPostMode,
  useSetReduceMode,
} from 'hooks/useOrderContext'
import { useWeb3Context } from 'hooks/useWeb3Context'
import { HoverMenuStyled } from 'layouts/app/NavSection'
import {
  bindHover,
  bindMenu,
  usePopupState,
} from 'material-ui-popup-state/hooks'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import {
  CurrentOrderContext,
  orderModeToName,
} from 'providers/CurrentOrderProvider'
import { useContext, useEffect, useState } from 'react'
import { MarketSymbol, OrderType, OrderTypeMap } from 'utils/globals'

import Banner from './Banner'

// TODO
// 1.) finish detailing various orderType types

export enum TabType {
  BUY,
  SELL,
}

const getButtonText = (
  hasAccount: boolean,
  symbol: MarketSymbol,
  tab: TabType,
) => {
  if (!hasAccount) {
    return 'Connect Wallet'
  } else {
    if (tab === TabType.BUY) {
      return `Long ${symbol.split('-')[0]}`
    } else {
      return `Short ${symbol.split('-')[0]}`
    }
  }
}

const OrderTypeInput = () => {
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setOrderType = useSetOrderType()
  return (
    <FormControl fullWidth variant="standard">
      <InputLabel>Order Type</InputLabel>
      <Select
        value={currentOrderState.orderType}
        label="Order type"
        onChange={(event) => {
          setOrderType(event?.target.value)
        }}
        defaultValue={OrderType.Market}
      >
        {Object.keys(OrderTypeMap).map((OrderTypeKey) => {
          return (
            <MenuItem value={OrderTypeKey} key={OrderTypeKey}>
              <Typography variant="h6">{OrderTypeKey}</Typography>
            </MenuItem>
          )
        })}
      </Select>
    </FormControl>
  )
}

const OrderPriceInput = () => {
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setOrderPrice = useSetOrderPrice()
  const setOrderSizeBase = useSetOrderSizeBase()
  const setOrderSizeQuote = useSetOrderSizeQuote()
  const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Market
  })
  const orderTypeLimit = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Limit
  })
  return (
    <TextField
      key={currentOrderState.orderPrice}
      InputProps={{
        sx: { fontSize: 14 },
        endAdornment: (
          <InputAdornment position="start">
            <Typography variant="h6">
              {currentOrderState.quoteSymbol}
            </Typography>
          </InputAdornment>
        ),
      }}
      fullWidth
      label={
        <Typography variant="h6">
          {orderTypeMarket || orderTypeLimit ? 'Price' : 'Trigger Price'}
        </Typography>
      }
      onChange={(ele) => {
        setOrderPrice(ele.target.value)
        setOrderSizeBase(currentOrderState.orderSizeBase)
        setOrderSizeQuote(currentOrderState.orderSizeQuote)
      }}
      variant="standard"
      defaultValue={currentOrderState.orderPrice}
      disabled={currentOrderState.orderType === orderTypeMarket}
    />
  )
}

const OrderSizeInput = () => {
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setOrderSizeBase = useSetOrderSizeBase()
  const setOrderSizeQuote = useSetOrderSizeQuote()
  const { data: userData } = useUserData()

  const {
    currentMarketState: { baseSymbol, quoteSymbol, marketSymbol, recentTrades },
  } = useContext(CurrentMarketContext)

  const latestPrice = recentTrades?.[0]?.price

  const setSizeToPercentage = (percent) => {
    console.log('calling setSizeToPercentage')
    const freeCollateral =
      userData?.totalCollateralUSD - userData?.usedCollateralUSD
    console.log('freeCollateral=', freeCollateral)
    console.log('currentOrderState=', currentOrderState)
    const orderBuffer = 0.01
    const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
      return OrderTypeMap[ele] == OrderType.Market
    })
    // set a scale to avoid hitting actual max
    console.log(
      'cx=',
      currentOrderState.orderType !== orderTypeMarket
        ? currentOrderState?.orderPrice
        : latestPrice,
    )
    const orderSize =
      ((1 - orderBuffer) * percent * freeCollateral) /
      (currentOrderState.orderType !== orderTypeMarket
        ? currentOrderState?.orderPrice
        : latestPrice)
    setOrderSizeBase(orderSize)
  }

  return (
    <Grid container direction="row">
      <Grid item xs={6} sx={{}}>
        <TextField
          InputProps={{
            sx: { fontSize: 14 },
            // start with ' ' to force placeholder visible..
            startAdornment: ' ',
            endAdornment: (
              <InputAdornment position="start">
                {currentOrderState.baseSymbol}
              </InputAdornment>
            ),
          }}
          fullWidth
          placeholder={'0.0'}
          type="string"
          label={'Size'}
          variant="standard"
          value={currentOrderState.orderSizeBaseFormatted}
          onChange={(event) => {
            setOrderSizeBase(event?.target?.value.replace(/,/g, ''))
          }}
        />
      </Grid>
      <Grid item xs={6} sx={{}}>
        <TextField
          InputProps={{
            sx: { fontSize: 14 },
            startAdornment: <InputAdornment position="start">≈</InputAdornment>,
            endAdornment: (
              <InputAdornment position="start">{'USD'}</InputAdornment>
            ),
            // endAdornment: (
            //   <InputAdornment position="start">{currentOrderState.quoteSymbol}</InputAdornment>
            // ),
          }}
          fullWidth
          placeholder={'0.0'}
          type="string"
          label={'Size'}
          variant="standard"
          value={currentOrderState.orderSizeQuoteFormatted}
          onChange={(event) => {
            setOrderSizeQuote(event?.target?.value.replace(/,/g, ''))
          }}
        />
      </Grid>
      <Grid item xs={12} sx={{ pt: 0.5 }}>
        <Grid container>
          <ButtonGroup
            fullWidth
            variant="outlined"
            style={{ height: 20 }}
            color="secondary"
            aria-label="outlined primary button group"
          >
            <Button
              sx={{ padding: 0 }}
              onClick={() => {
                setSizeToPercentage(0.1)
              }}
            >
              10%
            </Button>
            <Button
              sx={{ padding: 0 }}
              onClick={() => {
                setSizeToPercentage(0.25)
              }}
            >
              25%
            </Button>
            <Button
              sx={{ padding: 0 }}
              onClick={() => {
                setSizeToPercentage(0.5)
              }}
            >
              50%
            </Button>
            <Button
              sx={{ padding: 0 }}
              onClick={() => {
                setSizeToPercentage(0.75)
              }}
            >
              75%
            </Button>
            <Button
              sx={{ padding: 0 }}
              onClick={() => {
                setSizeToPercentage(1)
              }}
            >
              100%
            </Button>
          </ButtonGroup>
        </Grid>
      </Grid>
    </Grid>
  )
}

const OrderStopOut = () => {
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setIncludeTakeProfit = useSetIncludeTakeProfit()
  const setIncludeStopLoss = useSetIncludeStopLoss()
  const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Market
  })
  const orderTypeLimt = Object.keys(OrderTypeMap).find((ele) => {
    return OrderTypeMap[ele] == OrderType.Limit
  })
  return (
    <>
      {(currentOrderState.orderType === orderTypeMarket ||
        currentOrderState.orderType === orderTypeLimt) && (
        <Grid item xs={12} container direction="row">
          <Grid item sx={{ flexGrow: 1 }}>
            {' '}
            <FormControlLabel
              control={
                <Checkbox
                  checked={currentOrderState.includeTakeProfit}
                  onClick={() => {
                    setIncludeTakeProfit(!currentOrderState.includeTakeProfit)
                  }}
                  size="small"
                  sx={{ mr: -1 }}
                />
              }
              label={<Typography variant="caption">Add Take Profit</Typography>}
            />{' '}
          </Grid>
          <Grid item>
            {' '}
            <FormControlLabel
              control={
                <Checkbox
                  checked={currentOrderState.includeStopLoss}
                  onClick={() => {
                    setIncludeStopLoss(!currentOrderState.includeStopLoss)
                  }}
                  size="small"
                  sx={{ mr: -1 }}
                />
              }
              label={<Typography variant="caption">Add Stop Loss</Typography>}
            />
          </Grid>
        </Grid>
      )}
    </>
  )
}

interface OrderButtonProps {
  tab: TabType
  // baseSymbol: BaseSymbol
  setOpenLogin: React.Dispatch<React.SetStateAction<boolean>>
}
const OrderButton = ({ tab, setOpenLogin }: OrderButtonProps) => {
  const currentContext = useWeb3Context()
  const { currentOrderState } = useContext(CurrentOrderContext)
  const setReduceMode = useSetReduceMode()
  const setPostMode = useSetPostMode()
  const setOrderMode = useSetOrderMode()
  const popupState = usePopupState({
    variant: 'popover',
    popupId: 'OrderButtonMenu',
  })
  return (
    <Grid>
      <HoverMenuStyled {...bindMenu(popupState)}>
        <List sx={{ mt: -2, pb: -1.5 }}>
          {Object.keys(orderModeToName).map((orderMode) => {
            return (
              <ListItemButton
                key={`${orderMode}-button`}
                disabled={currentOrderState.orderMode === orderMode}
                onClick={(event) => {
                  setOrderMode(orderMode)
                }}
              >
                {orderModeToName[orderMode]}
              </ListItemButton>
            )
          })}
        </List>
      </HoverMenuStyled>
      <Button
        variant="contained"
        fullWidth
        sx={{
          color: 'black',
          backgroundColor: currentContext?.publicAddress
            ? tab === TabType.BUY
              ? 'tradeColors.bid'
              : 'tradeColors.ask'
            : undefined,
          '&:hover': {
            backgroundColor: currentContext?.publicAddress
              ? tab === TabType.BUY
                ? 'tradeColors.bidBright'
                : 'tradeColors.askBright'
              : undefined,
          },
        }}
        onClick={() => {
          !currentContext?.publicAddress && setOpenLogin(true)
        }}
      >
        {getButtonText(
          currentContext?.publicAddress,
          currentOrderState.baseSymbol || '',
          tab,
        )}
      </Button>
      <Grid item xs={12} container direction="row" sx={{ pt: 0 }}>
        <Grid item sx={{ flexGrow: 1 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={currentOrderState.reduceOnly}
                onClick={() => {
                  setReduceMode(!currentOrderState.reduceOnly)
                }}
                size="small"
                sx={{ mr: -1 }}
              />
            }
            label={<Typography variant="caption">Reduce</Typography>}
          />
        </Grid>
        <Grid item sx={{ flexGrow: 1 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={currentOrderState.postOnly}
                onClick={() => {
                  setPostMode(!currentOrderState.postOnly)
                }}
                size="small"
                sx={{ mr: -1 }}
              />
            }
            label={<Typography variant="caption">Post</Typography>}
          />
        </Grid>
        <Grid item {...bindHover(popupState)}>
          <Typography variant="caption" sx={{ mt: 0.85, mb: -0.85 }}>
            {currentOrderState.orderMode === 'untilCancel' &&
              'Good-till-cancel'}
            {currentOrderState.orderMode === 'immediateOrCancel' &&
              'Immediate-or-cancel'}
            {currentOrderState.orderMode === 'fillOrKill' && 'Fill-or-Kill'}
            <ArrowDropDownIcon sx={{ mt: 1, mb: -1 }} />
          </Typography>
        </Grid>
      </Grid>
    </Grid>
  )
}

const TradeSubmitterWidget = () => {
  const { currentOrderState } = useContext(CurrentOrderContext)
  const {
    currentMarketState: { baseSymbol, quoteSymbol, marketSymbol, recentTrades },
  } = useContext(CurrentMarketContext)

  const latestPrice = recentTrades?.[0]?.price

  const setOrderDetails = useSetOrderDetails()
  const setOrderPrice = useSetOrderPrice()
  const [openLogin, setOpenLogin] = useState(false)
  const [tab, setTab] = useState<TabType>(TabType.BUY)
  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setTab(newValue)
  }

  useEffect(() => {
    // TODO - investigate if latestPrice tickign is an issue
    setOrderDetails(marketSymbol, baseSymbol, quoteSymbol, latestPrice)
  }, [marketSymbol, baseSymbol, quoteSymbol])

  useEffect(() => {
    // TODO - investigate if latestPrice tickign is an issue
    if (!currentOrderState.estimatedPrice) {
      setOrderDetails(marketSymbol, baseSymbol, quoteSymbol, latestPrice)
    }
  }, [latestPrice])

  useEffect(() => {
    const orderTypeMarket = Object.keys(OrderTypeMap).find((ele) => {
      return OrderTypeMap[ele] == OrderType.Market
    })
    if (currentOrderState.orderType != orderTypeMarket) {
      setOrderPrice(latestPrice || 0)
    } else {
      setOrderPrice('MARKET')
    }
  }, [marketSymbol, currentOrderState.orderType])

  return (
    <>
      <LoginModal openModal={openLogin} setOpenModal={setOpenLogin} />
      <Grid container direction="column" sx={{ pr: 2, pl: 2 }}>
        <Grid item sx={{ pt: 1.5 }}>
          <Banner tab={tab} />
        </Grid>
        <Grid item>
          <Tabs value={tab} onChange={handleChange}>
            <Tab label="Long" style={{ minWidth: '50%' }} />
            <Tab label="Short" style={{ minWidth: '50%' }} />
          </Tabs>
        </Grid>
        <Grid item sx={{ pt: 1.5 }}>
          <OrderTypeInput />
        </Grid>
        <Grid item sx={{ pt: 1.5 }}>
          <OrderPriceInput />
        </Grid>
        <Grid item sx={{ pt: 1.5 }}>
          <OrderSizeInput />
        </Grid>
        <Grid item xs={12} sx={{ pt: 2.5 }}>
          <OrderButton tab={tab} setOpenLogin={setOpenLogin} />
        </Grid>
        <Grid item sx={{ mt: -2 }}>
          <OrderStopOut />
        </Grid>
      </Grid>
    </>
  )
}

export default TradeSubmitterWidget
