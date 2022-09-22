import { Grid } from '@mui/material'
import {} from 'react'
import { OrderSide } from 'utils/globals'

interface DepthVisualizerProps {
  depth: number
  orderType: OrderSide
}

const DepthVisualizer = ({ depth, orderType }: DepthVisualizerProps) => {
  return (
    <Grid container justifyContent="flex-end">
      <Grid
        data-testid="depth-visualizer"
        sx={{
          backgroundColor: `${
            orderType === OrderSide.BIDS
              ? 'tradeColors.bidBars'
              : 'tradeColors.askBars'
          }`,
        }}
        style={{
          height: '1.250em',
          width: `${depth}%`,
          marginTop: -24,
          position: 'relative',
          zIndex: 0,
        }}
      />
    </Grid>
  )
}

export default DepthVisualizer
