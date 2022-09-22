import { Theme } from '@mui/material/styles'

export default function Avatar() {
  return {
    MuiAvatar: {
      styleOverrides: {
        fallback: {
          height: '75%',
          width: '75%',
        },
      },
    },
  }
}
