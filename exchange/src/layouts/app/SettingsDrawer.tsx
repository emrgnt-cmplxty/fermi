import CloseIcon from '@mui/icons-material/Close'
import {
  Box,
  Divider,
  Drawer,
  FormControl,
  FormControlLabel,
  FormLabel,
  IconButton,
  Radio,
  RadioGroup,
  Stack,
  Typography,
} from '@mui/material'
import { useTheme } from '@mui/material/styles'
import Scrollbar from 'components/Scrollbar/index'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useEffect } from 'react'
import { useLocation } from 'react-router-dom'
import { ThemeKey } from 'theme/theme'

const DRAWER_WIDTH = 260

interface SettingsDrawerProps {
  isOpenSettingsDrawer: boolean
  onCloseSettingsDrawer: () => void
}

function SettingsDrawer({
  isOpenSettingsDrawer,
  onCloseSettingsDrawer,
}: SettingsDrawerProps) {
  const {
    settingsState: { theme: settingsTheme },
    settingsDispatch,
  } = useContext(SettingsContext)
  const { pathname } = useLocation()
  const theme = useTheme()

  useEffect(() => {
    if (isOpenSettingsDrawer) {
      onCloseSettingsDrawer()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname])

  const handleModeChanged = (
    _event: React.ChangeEvent<HTMLInputElement>,
    value: string,
  ) => {
    settingsDispatch({
      type: 'updateSetting',
      payload: {
        theme: value as ThemeKey,
      },
    })
  }

  const renderContent = (
    <Scrollbar
      sx={{
        height: 1,
        '& .simplebar-content': {
          height: 1,
          display: 'flex',
          flexDirection: 'column',
        },
      }}
    >
      <Box
        sx={{
          px: 2,
          py: 2,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <Typography variant="subtitle1">Settings</Typography>

        <IconButton onClick={onCloseSettingsDrawer}>
          <CloseIcon fontSize="small" />
        </IconButton>
      </Box>

      <Divider sx={{ borderStyle: 'dashed' }} />

      <Stack sx={{ p: 2 }} spacing={3}>
        <FormControl>
          <FormLabel id="mode-radio-buttons-label" style={{ color: 'gray' }}>
            {' '}
            Theme{' '}
          </FormLabel>
          <RadioGroup
            aria-labelledby="mode-radio-buttons-label"
            name="mode-radio-buttons-group"
            value={settingsTheme}
            onChange={handleModeChanged}
          >
            <FormControlLabel value="LIGHT" control={<Radio />} label="Light" />
            <FormControlLabel value="DARK" control={<Radio />} label="Dark" />
            {/*<FormControlLabel
              value="NATURE"
              control={<Radio />}
              label="Nature"
              />*/}
          </RadioGroup>
        </FormControl>
      </Stack>

      <Box sx={{ flexGrow: 1 }} />

      <Box sx={{ px: 2.5, pb: 3, mt: 10 }}>
        <Stack
          alignItems="center"
          spacing={3}
          sx={{ pt: 5, borderRadius: 2, position: 'relative' }}
        ></Stack>
      </Box>
    </Scrollbar>
  )

  return (
    <>
      <Drawer
        anchor="right"
        open={isOpenSettingsDrawer}
        onClose={onCloseSettingsDrawer}
        PaperProps={{
          sx: {
            width: DRAWER_WIDTH,
            borderBottomLeftRadius: 10,
            borderTopLeftRadius: 10,
          },
        }}
      >
        {renderContent}
      </Drawer>
    </>
  )
}

export default SettingsDrawer
