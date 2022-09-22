import { Theme } from '@mui/material/styles'

export default function LinearProgress() {
  return {
    MuiLinearProgress: {
      styleOverrides: {
        root: {
          borderRadius: 3,
          overflow: 'hidden',
        },
      },
    },
  }
}
