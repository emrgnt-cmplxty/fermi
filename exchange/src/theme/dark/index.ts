import { Shadows } from '@mui/material/styles/shadows'

const components = {
  MuiTableCell: {
    styleOverrides: {
      root: {
        borderBottom: '1px solid rgba(145, 158, 171, 0.24)',
      },
    },
  },
  MuiTableRow: {
    styleOverrides: {
      root: {
        '&:last-child td': {
          borderBottom: 0,
        },
      },
    },
  },
}
// primary color candidates
// orange - F2AA4CFF
// dark green - 006B38FF
// cherry tomato - E94B3CFF
const palette = {
  mode: 'dark',
  tradeColors: {
    bidBright: '#76ff03',
    bidBars: '#113534',
    bid: '#22dd8f',
    askBars: '#3d1e28',
    askBright: '#EE4B2B',
    ask: 'red',
  },
  primary: {
    contrastText: '#ffffff',
    main: '#74FBE6',
    hover: '#29eecb',
    light: '#000000',
  },
  secondary: {
    contrastText: '#919eab',
    main: '#919eab',
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
    main: '#000000',
  },
  gradients: {
    background: 'linear-gradient(to  bottom, #171F2B, #19172B)',
    lendRewards: 'linear-gradient(to  right, #04245c, #1e4382 )',
    aprCard: 'linear-gradient(rgb(30, 67, 130), rgb(30, 67, 130))',
    lendRow: 'linear-gradient(rgb(23, 31, 43), rgb(25, 23, 43))',
    poolTierCard: 'linear-gradient(to  bottom, #3f4a59, #19172B)',
    slider: 'linear-gradient(to left, rgb(20, 120, 100), rgb(100, 6, 5))',
  },
  divider: 'rgba(145, 158, 171, 0.24)',
  text: {
    primary: '#ffffff',
    secondary: '#919eab',
    active: '#6163ff',
    // navbar
    navbar: '#ffffff',
  },
  background: {
    default: '#000000',
    component: '#101820',
    navbar: '#101820',
    paper: '#21262E',
    navmenu: '#000000',
  },
}

export const shadows: Shadows = [
  'none',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 3px 4px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 2px 2px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 3px 4px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 3px 4px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 4px 6px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 4px 6px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 4px 8px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 5px 8px -2px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 6px 12px -4px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 7px 12px -4px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 6px 16px -4px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 7px 16px -4px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 8px 18px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 9px 18px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 10px 20px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 11px 20px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 12px 22px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 13px 22px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 14px 24px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 16px 28px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 18px 30px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 20px 32px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 22px 34px -8px rgba(0,0,0,0.50)',
  '0 0 1px 0 rgba(0,0,0,0.70), 0 24px 36px -8px rgba(0,0,0,0.50)',
]

const darkTheme = {
  components,
  palette,
  shadows,
}

export default darkTheme
