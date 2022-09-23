import { Grid, Tab, Tabs, Typography } from '@mui/material'
import { StyledHeader } from 'components/TradeDashboard'
import WidgetCloseIcon from 'components/WidgetCloseIcon'
import { useUserData } from 'hooks/react-query/useUser'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useState } from 'react'

import AdvancedOrders from './AdvancedOrders'
import OrderHistory from './OrderHistory'
import Orders from './Orders'
import Positions from './Positions'

interface HeaderContentProps {
  tab: number
  setTab: React.Dispatch<React.SetStateAction<number>>
}

const HeaderContent = ({ tab, setTab }: HeaderContentProps) => {
  const [isActive, setIsActive] = useState(false)
  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setTab(newValue)
  }
  const { data: userData } = useUserData({ refetchInterval: 1000 })

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
      <Tabs value={tab} onChange={handleChange}>
        <Tab
          label={
            <Typography variant="h7">
              {' '}
              {`Positions (${userData?.positions.length})`}{' '}
            </Typography>
          }
          style={{ maxWidth: 250 }}
        />
        <Tab
          label={
            <Typography variant="h7">
              {' '}
              {`Limit Orders (${userData?.openLimitOrders.length})`}{' '}
            </Typography>
          }
          style={{ maxWidth: 250 }}
        />
        <Tab
          label={
            <Typography variant="h7">
              {' '}
              {`Advanced Orders (${userData?.openAdvancedOrders.length})`}{' '}
            </Typography>
          }
          style={{ maxWidth: 250 }}
        />
        <Tab
          label={<Typography variant="h7"> Order History </Typography>}
          style={{ maxWidth: 250 }}
        />
        <Tab
          label={<Typography variant="h7"> Trade History </Typography>}
          style={{ maxWidth: 250 }}
        />
      </Tabs>
      <StyledHeader className="header" sx={{ flexGrow: 1 }} />
      <Grid item style={{ width: 20 }}>
        {isActive && <WidgetCloseIcon />}
      </Grid>
    </Grid>
  )
}

const PortfolioWidget = () => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)

  const [tab, setTab] = useState(0)
  const { data: userData } = useUserData({ refetchInterval: 1000 })

  return (
    <>
      <HeaderContent tab={tab} setTab={setTab} />
      {tab === 0 && <Positions />}
      {tab === 1 && <Orders header="" />}
      {tab === 2 && <AdvancedOrders header="" />}
      {tab === 3 && (
        <OrderHistory input={userData?.orderHistory} header="Order History" />
      )}
      {tab === 4 && (
        <OrderHistory
          input={userData?.orderHistory?.filter((ele) => {
            return ele.status !== 'Canceled'
          })}
          header="Order History"
        />
      )}
    </>
  )
}

export default PortfolioWidget
