import { Box, Card, Divider, Tab, Tabs } from '@mui/material'
import { DisplayBox } from 'components/DisplayBox/DisplayBox'
import PortfolioWidget from 'components/PortfolioWidget'
import Highcharts from 'highcharts/highstock'
import HighchartsReact from 'highcharts-react-official'
import { useState } from 'react'

import styles from './AccountDashboard.module.scss'
import { PositionsTable } from './PositionsTable'
import { WalletTable } from './WalletTable'

export const AccountDashboard = () => {
  const [tab, setTab] = useState('Positions')
  const options = {
    chart: {
      type: 'spline',
      backgroundColor: '#101820',
    },
    title: {
      text: '',
    },
    xAxis: {
      categories: [
        'Thu, 2PM',
        'Fri, 7AM',
        'Thu, 7AM',
        'Fri, 7AM',
        'Thu, 7AM',
        'Fri, 7AM',

        'Thu, 7AM',

        'Fri, 7AM',

        'Thu, 7AM',

        'Fri, 7AM',

        'Thu, 7AM',
      ],
    },
    yAxis: {
      title: {
        text: '',
      },
      labels: {
        enabled: false,
      },
      visible: false,
    },

    plotOptions: {
      series: {
        marker: {
          enabled: false,
        },
      },
    },
    credits: {
      enabled: false,
    },
    series: [
      {
        showInLegend: false,
        data: [1, 2, 1, 4, 3, 6, 2, 1, 4, 3, 6],
        color: '#74FBE6',
      },
    ],
  }

  return (
    <Box display="flex" flexDirection="column" width="100%">
      <Box display="flex" width="100%">
        <Card className={styles.overview}>
          <Box display="flex" flexDirection="column">
            <DisplayBox
              title={'Portfolio Value'}
              data={'$564.93'}
              metaText={'$564.93'}
              primaryVariant="h4"
            />
            <Divider />
            <Box display="inline-flex">
              <DisplayBox title={'Margin Usage'} data={'$564.93'} />
              <Divider orientation="vertical" flexItem />
              <DisplayBox title={'Free Collateral'} data={'$564.93'} />
            </Box>
            <Divider />
            <Box display="inline-flex">
              <DisplayBox title={'Leverage'} data={'$564.93'} />
              <Divider orientation="vertical" flexItem />
              <DisplayBox title={'Buying Power'} data={'$564.93'} />
            </Box>
          </Box>
        </Card>
        <Box display="flex" width="100%">
          <HighchartsReact highcharts={Highcharts} options={options} />{' '}
        </Box>
      </Box>
      <PortfolioWidget />

      {/* <Tabs
        value={tab}
        onChange={(event, value) => {
          setTab(value)
        }}
        className={styles.tabs}
      >
        <Tab label="Positions" value={'Positions'} />
        <Tab label="Wallet" value={'Wallet'} />
      </Tabs>
      {tab === 'Positions' && <PositionsTable />}
      {tab === 'Wallet' && <WalletTable />} */}
    </Box>
  )
}
