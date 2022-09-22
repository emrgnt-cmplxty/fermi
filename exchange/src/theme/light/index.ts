import { Shadows } from '@mui/material/styles/shadows'

const components = {
  MuiInputBase: {
    styleOverrides: {
      input: {
        '&::placeholder': {
          opacity: 0.86,
          color: '#42526e',
        },
      },
    },
  },
}

const palette = {
  mode: 'light',
  status: {
    danger: '#e53e3e',
  },
  tradeColors: {
    bidBars: '#4caf50',
    bidBright: '#76ff03',
    bid: 'green',
    askBars: '#f44336',
    askBright: '#EE4B2B',
    ask: 'red',
  },
  primary: {
    contrastText: '#ffffff',
    main: '#3A7D73',
    hover: '#3A7D73',
    light: '#D3D3D3',
  },
  secondary: {
    contrastText: '#ffffff',
    main: '#000000',
  },
  success: {
    contrastText: '#ffffff',
    main: '#4caf50',
  },
  warning: {
    contrastText: '#ffffff',
    main: '#ff9800',
  },
  error: {
    contrastText: '#ffffff',
    main: '#f44336',
  },
  buttonText: {
    main: '#ffffff',
  },
  gradients: {
    background: 'linear-gradient(to  bottom, #f9faf8, #ffffff)',
    lendRewards: 'linear-gradient(to  right, #1c82d4, #2196f3)',
    aprCard: 'linear-gradient(rgb(3, 169, 244), rgb(28, 130, 212))',
    lendRow: 'linear-gradient(rgb(249, 250, 248), rgb(255, 255, 255))',
    poolTierCard: 'linear-gradient(to  bottom, #f9faf8, #ffffff)',
  },
  text: {
    primary: '#000000',
    secondary: '#6b778c',
    // for navbar
    navbar: '#000000',
  },
  background: {
    default: '#EAEEEF',
    component: '#ffffff',
    // navbar is tinted by mui alpha
    navbar: '#ffffff',
    paper: '#f4f5f7',
    navmenu: '#ffffff',
  },
  action: {
    active: '#6b778c',
  },
}

export const shadows: Shadows = [
  'none',
  '0px 1px 2px rgba(0, 0, 0, 0.12), 0px 0px 0px 1px rgba(0, 0, 0, 0.05)',
  '0px 2px 4px rgba(0, 0, 0, 0.15), 0px 0px 0px 1px rgba(0, 0, 0, 0.05)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 3px 4px -2px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 3px 4px -2px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 4px 6px -2px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 4px 6px -2px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 4px 8px -2px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 5px 8px -2px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 6px 12px -4px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 7px 12px -4px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 6px 16px -4px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 7px 16px -4px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 8px 18px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 9px 18px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 10px 20px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 11px 20px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 12px 22px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 13px 22px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 14px 24px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 16px 28px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 18px 30px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 20px 32px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 22px 34px -8px rgba(0,0,0,0.25)',
  '0 0 1px 0 rgba(0,0,0,0.31), 0 24px 36px -8px rgba(0,0,0,0.25)',
]

const lightTheme = {
  components,
  palette,
  shadows,
}

export default lightTheme
