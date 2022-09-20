import { Grid, Tab, Tabs, useMediaQuery } from '@mui/material'
import { useTheme } from '@mui/material/styles'
import MarketTable from 'components/MarketTable'
import { useState } from 'react'
import { MarketType } from 'utils/globals'

const tabToTypes: { [value: string]: MarketType } = {
  //0: 'spot',
  0: 'futures',
  3: 'favorites',
}

function MarketsPage() {
  const theme = useTheme()
  const [tab, setTab] = useState(0)
  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setTab(newValue)
  }

  const matchUpXl = useMediaQuery(theme.breakpoints.up('lg'))
  return (
    <Grid container justifyContent={matchUpXl ? 'center' : 'flex-start'}>
      <Grid sx={{ maxWidth: matchUpXl ? 1500 : undefined }}>
        <Tabs value={tab} onChange={handleChange}>
          {/* <Tab label="Spot" style={{ fontSize: 20, maxWidth: '25%' }} /> */}
          <Tab label="Futures" style={{ fontSize: 20, maxWidth: '25%' }} />
          <Tab
            label="Leveraged Tokens"
            style={{ fontSize: 20, maxWidth: '25%' }}
            disabled={true}
          />
          <Tab
            label="Vaults"
            style={{ fontSize: 20, maxWidth: '25%' }}
            disabled={true}
          />
          <Tab label="Favorites" style={{ fontSize: 20, maxWidth: '25%' }} />
        </Tabs>
        <MarketTable type={tabToTypes[tab]} />
      </Grid>
    </Grid>
  )
}

export default MarketsPage
