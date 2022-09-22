import { styled } from '@mui/material/styles'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useEffect, useState } from 'react'
import { Outlet } from 'react-router-dom'

//
import Navbar from './Navbar'
import SettingsDrawer from './SettingsDrawer'
import Sidebar from './Sidebar'

const APP_BAR_MOBILE = 64
const APP_BAR_DESKTOP = 92

const RootStyled = styled('div')({
  display: 'flex',
  minHeight: '100%',
  overflow: 'hidden',
})

const MainStyled = styled('div')(({ theme }) => ({
  flexGrow: 1,
  overflow: 'auto',
  minHeight: '100%',
  paddingTop: APP_BAR_MOBILE + 24,
  paddingBottom: theme.spacing(10),
  [theme.breakpoints.up('lg')]: {
    paddingTop: APP_BAR_DESKTOP + 24,
    paddingLeft: theme.spacing(2),
    paddingRight: theme.spacing(2),
  },
}))

function AppLayout() {
  const { settingsState } = useContext(SettingsContext)
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [settingsDrawerOpen, setSettingsDrawerOpen] = useState(false)

  useEffect(() => {
    document.body.className = settingsState.theme || ''
  }, [settingsState.theme])

  return (
    <RootStyled className={settingsState.theme}>
      <Navbar
        onOpenSidebar={() => setSidebarOpen(true)}
        onOpenSettingsDrawer={() => setSettingsDrawerOpen(true)}
      />
      <Sidebar
        isOpenSidebar={sidebarOpen}
        onCloseSidebar={() => setSidebarOpen(false)}
      />
      <SettingsDrawer
        isOpenSettingsDrawer={settingsDrawerOpen}
        onCloseSettingsDrawer={() => setSettingsDrawerOpen(false)}
      />
      <MainStyled>
        <Outlet />
      </MainStyled>
    </RootStyled>
  )
}

export default AppLayout
