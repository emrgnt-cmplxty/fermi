import {
  Avatar,
  Box,
  Divider,
  Drawer,
  Grid,
  Typography,
  useMediaQuery,
} from '@mui/material'
import { useTheme } from '@mui/material/styles'
import Scrollbar from 'components/Scrollbar/index'
import { useEffect } from 'react'
import { Link as RouterLink, useLocation } from 'react-router-dom'

const DRAWER_WIDTH = 280

const SidebarContent = () => {
  return (
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
      <Grid container direction="column">
        <Grid item>
          <Grid
            container
            direction="column"
            sx={{
              pt: 3,
              pb: 2,
              alignItems: 'center',
            }}
          >
            <RouterLink to="/">
              <Avatar
                alt="logo"
                src="https://www.gitbook.com/cdn-cgi/image/width40,height=40,fit=contain,dpr=2,format=auto/https%3A%2F%2F3211716736-files.gitbook.io%2F~%2Ffiles%2Fv0%2Fb%2Fgitbook-legacy-files%2Fo%2Fspaces%252F-MhRebXhAYZO31eq2-Wx%252Favatar-1630029845429.png%3Fgeneration%3D1630029845683791%26alt%3Dmedia"
                variant="square"
              />
            </RouterLink>

            <Typography variant="h5" component="div">
              DMEX
            </Typography>
          </Grid>
        </Grid>{' '}
      </Grid>

      <Divider />
      <Box flexGrow="1">
        <Box
          width="100%"
          display="flex"
          justifyContent="space-evenly"
          sx={{ position: 'absolute', bottom: 0, pb: 2 }}
        ></Box>
      </Box>
    </Scrollbar>
  )
}
interface SidebarProps {
  isOpenSidebar: boolean
  onCloseSidebar: () => void
}

function Sidebar({ isOpenSidebar, onCloseSidebar }: SidebarProps) {
  const { pathname } = useLocation()
  const theme = useTheme()
  const matchUpLg = useMediaQuery(theme.breakpoints.up('lg'))

  useEffect(() => {
    if (isOpenSidebar) {
      onCloseSidebar()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname])

  return (
    !matchUpLg && (
      <Drawer
        open={isOpenSidebar}
        onClose={onCloseSidebar}
        PaperProps={{
          sx: { width: DRAWER_WIDTH },
        }}
      >
        <SidebarContent />
      </Drawer>
    )
  )
}

export default Sidebar
