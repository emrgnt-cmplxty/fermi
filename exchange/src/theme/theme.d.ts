import * as material from '@mui/material'

declare module '@mui/material' {
  interface Color {
    500_8?: string
    500_12?: string
    500_16?: string
    500_24?: string
    500_32?: string
    500_48?: string
    500_56?: string
    500_80?: string
  }
  interface Theme {
    customShadows: {
      z1: string
      z8: string
      z12: string
      z16: string
      z20: string
      z24: string
      primary: string
      secondary: string
      info: string
      success: string
      warning: string
      error: string
    }
  }
  // allow configuration using `createTheme`
  interface ThemeOptions {
    link?: {
      [key: string]: string
    }
    gradients?: {
      [key: string]: string
    }
    customShadows?: {
      z1?: string
      z8?: string
      z12?: string
      z16?: string
      z20?: string
      z24?: string
      primary?: string
      secondary?: string
      info?: string
      success?: string
      warning?: string
      error?: string
    }
  }
}

export type ThemeKey = 'LIGHT' | 'DARK' | 'NATURE'
