import { CssBaseline, ThemeOptions } from '@mui/material'
import {
  createTheme,
  StyledEngineProvider,
  ThemeProvider as MUIThemeProvider,
} from '@mui/material/styles'
import { RTL } from 'components/RTL/index'
import { merge } from 'lodash'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useMemo } from 'react'
import { ReactNode } from 'react'
import { baseThemeOptions, themes } from 'theme'
import componentsOverride from 'theme/overrides'

interface ThemeProviderProps {
  children: ReactNode
}

function ThemeProvider({ children }: ThemeProviderProps) {
  const { settingsState, isLoading } = useContext(SettingsContext)

  const settingsTheme = useMemo(
    () => (settingsState.theme ? themes[settingsState.theme] : undefined),
    [settingsState.theme],
  )

  // TODO: properly type ThemeOptions
  const theme = createTheme(
    merge(baseThemeOptions as ThemeOptions, settingsTheme, {
      direction: settingsState.direction,
    }),
  )
  theme.components = componentsOverride(theme)

  if (isLoading) {
    return null
  }

  return (
    <StyledEngineProvider injectFirst>
      <MUIThemeProvider theme={theme}>
        <RTL direction={settingsState.direction}>
          <CssBaseline />
          {children}
        </RTL>
      </MUIThemeProvider>
    </StyledEngineProvider>
  )
}

export default ThemeProvider
