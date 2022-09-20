import { Grid, Tab, Tabs, useMediaQuery } from '@mui/material'
import { useTheme } from '@mui/material/styles'
import MarketTable from 'components/MarketTable'
import { AccountDashboard } from 'components/Portfolio/AccountDashboard'
import { PLDashboard } from 'components/Portfolio/PLDashboard'
import { useState } from 'react'
import { MarketType } from 'utils/globals'

const tabToTypes: { [value: string]: MarketType } = {
  0: 'dashboard',
  1: 'futures',
  2: 'favorites',
}

function PortfolioPage() {
  const theme = useTheme()
  const [tab, setTab] = useState(0)
  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setTab(newValue)
  }

  const matchUpXl = useMediaQuery(theme.breakpoints.up('lg'))
  return (
    <Grid container justifyContent={matchUpXl ? 'center' : 'flex-start'}>
      <Grid sx={{ width: '80%' }}>
        <Tabs value={tab} onChange={handleChange}>
          <Tab label="Dashboard" style={{ fontSize: 20, maxWidth: '25%' }} />
          <Tab label="P&L Analysis" style={{ fontSize: 20, maxWidth: '25%' }} />
          <Tab label="Favorites" style={{ fontSize: 20, maxWidth: '25%' }} />
        </Tabs>
        {tab === 0 && <AccountDashboard />}
        {tab === 1 && <PLDashboard />}
      </Grid>
    </Grid>
  )
}

export default PortfolioPage
