import { Theme, ThemeOptions } from '@mui/material'
import { createTheme } from '@mui/material/styles'
import { merge } from 'lodash'

import darkTheme from './dark'
import lightTheme from './light'
import componentsOverride from './overrides'
import palette from './palette'
import shadows, { customShadows } from './shadows'
import { ThemeKey } from './theme'
import typography from './typography'

export const THEME_KEYS: ThemeKey[] = ['LIGHT', 'DARK']

export const baseThemeOptions = {
  palette,
  shape: { borderRadius: 8 },
  typography,
  shadows,
  customShadows,
}

export const themes = {
  LIGHT: { ...lightTheme },
  DARK: { ...darkTheme },
}

interface AugmentedThemes {
  [themeName: string]: Theme & { spacingValue: string }
}

export const getAugmentedThemes = (): AugmentedThemes => {
  return THEME_KEYS.reduce((augmentedThemes: AugmentedThemes, themeKey) => {
    const theme = themes[themeKey]

    const augmentedTheme = createTheme(
      merge(baseThemeOptions as ThemeOptions, theme),
    )
    augmentedTheme.components = componentsOverride(augmentedTheme)

    augmentedThemes[themeKey] = {
      ...augmentedTheme,
      spacingValue: augmentedTheme.spacing(1),
    }
    return augmentedThemes
  }, {})
}
