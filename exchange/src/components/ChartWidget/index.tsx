import { Button, Divider, Grid, Typography } from '@mui/material'
import { StyledHeader } from 'components/TradeDashboard'
import WidgetCloseIcon from 'components/WidgetCloseIcon'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { useContext, useState } from 'react'

import DepthChart from './DepthOfMarketChart'
import TVChart from './TVChart'

interface HeaderContentProps {
  selected: string
  setSelected: React.Dispatch<React.SetStateAction<string>>
}
const HeaderContent = ({ selected, setSelected }: HeaderContentProps) => {
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
      <Grid item sx={{ pl: 2 }}>
        <Typography variant="h6" sx={{ mt: 1 }}>
          Chart
        </Typography>
      </Grid>
      <StyledHeader className="header" sx={{ flexGrow: 1 }} />
      {/* <Grid item sx={{mr:20, pt: -10}} style={{top: "-100px"}}> */}
      <Grid item>
        <Grid container justifyContent="flex-end">
          <Button
            color={selected === 'TradeView' ? 'primary' : 'secondary'}
            onClick={() => {
              setSelected('TradeView')
            }}
          >
            TradeView
          </Button>
          <Button
            color={selected === 'Depth' ? 'primary' : 'secondary'}
            onClick={() => {
              setSelected('Depth')
            }}
          >
            Depth
          </Button>
        </Grid>
      </Grid>
      <Grid item style={{ width: 20 }}>
        {isActive && <WidgetCloseIcon />}
      </Grid>
    </Grid>
  )
}

interface ChartWidgetProps {
  height: number
  width: number
}

const ChartWidget = ({ height, width }: ChartWidgetProps) => {
  const {
    currentMarketState: { marketSymbol },
  } = useContext(CurrentMarketContext)
  const [selected, setSelected] = useState<'TradeView' | 'Depth'>('TradeView')

  return (
    <Grid container direction="column">
      <HeaderContent selected={selected} setSelected={setSelected} />
      <Divider />
      {selected === 'TradeView' && (
        <Grid item>
          <TVChart
            height={height}
            width={width}
            /// assume everything is trading against USD if spot
            symbol={marketSymbol}
          />
        </Grid>
      )}
      {selected === 'Depth' && (
        <Grid item>
          <DepthChart
            height={height}
            width={width}
            /// assume everything is trading against USD if spot
            symbol={marketSymbol}
          />
        </Grid>
      )}
    </Grid>
  )
}

export default ChartWidget
