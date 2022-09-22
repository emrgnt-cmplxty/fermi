import CalculateIcon from '@mui/icons-material/Calculate'
import { Button, Grid, Typography } from '@mui/material'
import LeverageModal from 'components/LeverageModal'
import { useUserData } from 'hooks/react-query/useUser'
import { useState } from 'react'

import { TabType } from './index'

interface BannerWidgetProps {
  tab: TabType.BUY | TabType.SELL
}

const BannerWidget = ({ tab }: BannerWidgetProps) => {
  const { data: userData } = useUserData()
  const [openLeverage, setOpenLeverage] = useState(false)
  return (
    <>
      <LeverageModal openModal={openLeverage} setOpenModal={setOpenLeverage} />
      <Grid container justifyContent="center" alignItems="center">
        <Grid item style={{ flexGrow: 1 }}>
          <Button
            variant="outlined"
            style={{ height: 25, width: 75, borderRadius: 4, padding: 10 }}
            sx={{ mr: 1 }}
            onClick={() => {
              setOpenLeverage(true)
            }}
          >
            <Typography variant="caption">Cross</Typography>
          </Button>
          <Button
            variant="outlined"
            sx={{
              mr: 1,
              color: `tradeColors.${tab == TabType.BUY ? 'bid' : 'ask'}`,
              borderColor: `tradeColors.${tab == TabType.BUY ? 'bid' : 'ask'}`,
            }}
            style={{ height: 25, borderRadius: 4, padding: 10 }}
            onClick={() => {
              setOpenLeverage(true)
            }}
          >
            <Typography variant="caption">
              {tab == TabType.BUY
                ? `Long ${userData?.longMaxLeverage}x`
                : `Short ${userData?.shortMaxLeverage}x`}
            </Typography>
          </Button>
        </Grid>
        <Grid item sx={{ mt: 0.5, mb: -0.5, mr: 1 }}>
          <CalculateIcon />
        </Grid>
      </Grid>
    </>
  )
}

export default BannerWidget
