import { Grid } from '@mui/material'
// import { DepthChart as DepthChartWrapper } from "pennant";
// import "pennant/dist/style.css";
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useEffect, useState } from 'react'

import { DepthChart as DepthChartWrapper } from './depth-chart'

interface DepthChartProps {
  height: number
  width: number
  symbol: string
}

const DepthChart = ({ height, width, symbol }: DepthChartProps) => {
  const { settingsState } = useContext(SettingsContext)
  const {
    currentMarketState: { rawAsks, rawBids },
  } = useContext(CurrentMarketContext)
  return (
    <Grid container style={{ height: height - 100, width: width }}>
      <DepthChartWrapper
        data={{ asks: rawAsks.slice(0, 25), bids: rawBids.slice(0, 25) }}
        theme={settingsState.theme === 'LIGHT' ? 'light' : 'dark'}
      />
    </Grid>
  )
}
export default DepthChart
