import { Settings } from '@mui/icons-material'
import MenuIcon from '@mui/icons-material/Menu'
import {
  alpha,
  AppBar,
  Box,
  Divider,
  Grid,
  IconButton,
  Link,
  Stack,
  styled,
  Toolbar,
  Typography,
} from '@mui/material'
import LanguageCCYSettings from 'components/LanguageCCYSettings'
import { WalletButton } from 'components/WalletButton'
import { useWeb3Context } from 'hooks/useWeb3Context'
import HorizNavSection, { HoverMenuStyled } from 'layouts/app/NavSection'
import {
  bindHover,
  bindMenu,
  usePopupState,
} from 'material-ui-popup-state/hooks'
import { SettingsContext } from 'providers/SettingsProvider'
import { ReactElement, useContext } from 'react'
import { useLocation } from 'react-router-dom'

import navSectionItems from './navSectionItems'

const APPBAR_MOBILE = 48
const APPBAR_DESKTOP = 40

const RootStyled = styled(AppBar)(({ theme }) => ({
  boxShadow: 'none',
  backdropFilter: 'blur(6px)',
  WebkitBackdropFilter: 'blur(6px)', // Fix on Mobile
  backgroundColor: alpha(theme.palette.background.navbar, 0.72),
}))

const ToolbarStyled = styled(Toolbar)(({ theme }) => ({
  height: APPBAR_MOBILE,
  [theme.breakpoints.up('lg')]: {
    height: APPBAR_DESKTOP,
    padding: theme.spacing(0, 5),
  },
}))

interface NavbarProps {
  onOpenSidebar: () => void
  onOpenSettingsDrawer: () => void
}

function Navbar({
  onOpenSidebar,
  onOpenSettingsDrawer,
}: NavbarProps): ReactElement {
  const { settingsState } = useContext(SettingsContext)
  const popupState = usePopupState({
    variant: 'popover',
    popupId: 'NavbarMenu',
  })
  const currentContext = useWeb3Context()
  const { pathname } = useLocation()

  return (
    <RootStyled style={{ height: 60, minHeight: 60 }}>
      <ToolbarStyled style={{ height: 60, minHeight: 60 }}>
        <IconButton
          onClick={onOpenSidebar}
          sx={{ mr: 1, color: 'text.primary', display: { lg: 'none' } }}
        >
          <MenuIcon />
        </IconButton>
        <HorizNavSection items={navSectionItems} />
        <Box sx={{ flexGrow: 1 }} />
        <Stack
          direction="row"
          alignItems="center"
          spacing={{ xs: 0.5, sm: 1.5 }}
        >
          <Box display="flex">
            {currentContext?.connected && (
              <Typography
                component={Link}
                href={'/app/portfolio/dashboard'}
                sx={{ mt: 1, mr: 2 }}
                variant="h6"
                underline="none"
                color={
                  pathname.toLowerCase().includes('portfolio')
                    ? 'active'
                    : 'textPrimary'
                }
              >
                Portfolio
              </Typography>
            )}
            <WalletButton />
          </Box>
          <Grid
            container
            direction="row"
            {...bindHover(popupState)}
            sx={{ color: 'text.navbar' }}
          >
            <Typography>{settingsState?.language}</Typography>
            <Divider
              sx={{
                backgroundColor: 'text.navbar',
                width: 1.1,
                height: 20,
                ml: 1,
                mr: 1,
              }}
              orientation="vertical"
            />
            <Typography>{settingsState?.currency}</Typography>
          </Grid>
          <HoverMenuStyled {...bindMenu(popupState)}>
            <LanguageCCYSettings />
          </HoverMenuStyled>
          <IconButton
            sx={{ width: 30, height: 30, color: 'text.navbar' }}
            onClick={onOpenSettingsDrawer}
          >
            <Settings />
          </IconButton>
        </Stack>
      </ToolbarStyled>
      <Divider />
    </RootStyled>
  )
}

export default Navbar
