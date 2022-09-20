import {
  Box,
  Button,
  Divider,
  Grid,
  LinearProgress,
  Typography,
} from '@mui/material'
import { StyledHeader } from 'components/TradeDashboard'
import WidgetCloseIcon from 'components/WidgetCloseIcon'
import { useUserData } from 'hooks/react-query/useUser'
import { useState } from 'react'
import { rgbToHex } from 'utils/formatters'

const HeaderContent = () => {
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
      <Grid item sx={{ pl: 1, pt: 0.75, pb: 0.8 }}>
        <Typography variant="h7">Account</Typography>
      </Grid>
    </Grid>
  )
}

const UserHealthWidget = () => {
  const { data: userData } = useUserData()

  const health =
    (100 * userData?.usedCollateralUSD) / userData?.totalCollateralUSD
  const AMT_RED = parseInt(String(20 + 2 * health))
  const AMT_GREEN = parseInt(String(200 - 1.2 * health))
  const AMT_BLUE = parseInt(String(100 - health))
  // green -> red & health = 100
  const startVal = rgbToHex(20, 120, 100)
  const endVal = rgbToHex(AMT_RED, AMT_GREEN, AMT_BLUE)

  return (
    <Grid>
      <HeaderContent />
      <Divider />
      <Grid container direction="column" sx={{ pr: 2, pl: 2, pt: 1 }}>
        <Grid item>
          <Typography variant="h7">
            {`Margin Utilization: ${health.toFixed(2)}%`}
          </Typography>
        </Grid>
        <Grid item sx={{ mb: 2 }}>
          <Box position="relative" mt={1}>
            <LinearProgress
              sx={{
                backgroundColor: `background.paper`,
                '& .MuiLinearProgress-bar': {
                  background: `linear-gradient(to right bottom, ${startVal}, ${endVal})`,
                },
              }}
              variant="determinate"
              value={health}
            />
          </Box>
        </Grid>
        <Grid item>
          <Grid container direction="row">
            <Grid item sx={{ flexGrow: 1 }}>
              <Grid container direction="row">
                <Grid item>
                  <Grid container direction="column">
                    <Grid item>
                      <Typography variant="h7">Used Margin</Typography>
                    </Grid>
                    <Grid item>
                      <Typography variant="h7">
                        {`$${userData?.usedCollateralUSD}`}
                      </Typography>
                    </Grid>
                  </Grid>
                </Grid>
              </Grid>
            </Grid>
            <Grid item>
              <Grid container direction="row">
                <Grid item>
                  <Grid container direction="column">
                    <Grid item>
                      <Typography variant="h7">Total. Margin</Typography>
                    </Grid>
                    <Grid item>
                      <Typography variant="h7">
                        {`$${userData?.totalCollateralUSD}`}
                      </Typography>
                    </Grid>
                  </Grid>
                </Grid>
              </Grid>
            </Grid>
          </Grid>
        </Grid>
        <Grid item sx={{ pt: 0.5, pb: 1 }}>
          <Grid container direction="row">
            <Grid item xs={4} sx={{ pr: 0.5 }}>
              <Button
                fullWidth
                color="primary"
                variant="outlined"
                sx={{ padding: 0.5, minHeight: 30 }}
              >
                <Typography variant="h7" color="success">
                  {' '}
                  Deposit{' '}
                </Typography>
              </Button>
            </Grid>
            <Grid item xs={4} sx={{ pl: 0.5, pr: 0.5 }}>
              <Button
                fullWidth
                variant="outlined"
                sx={{
                  color: 'gray',
                  borderColor: 'gray',
                  padding: 0.5,
                  minHeight: 30,
                }}
              >
                <Typography variant="h7"> Withdraw </Typography>
              </Button>
            </Grid>
            <Grid item xs={4} sx={{ pl: 0.5 }}>
              <Button
                fullWidth
                color="success"
                variant="outlined"
                sx={{ padding: 0.5, minHeight: 30 }}
              >
                <Typography variant="h7"> Buy </Typography>
              </Button>
            </Grid>
          </Grid>
        </Grid>
      </Grid>
    </Grid>
  )
}

export default UserHealthWidget
