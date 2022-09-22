import { Theme } from '@mui/material/styles'

export default function Button(theme: Theme) {
  return {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 4,
          // '&:hover': {
          //   //boxShadow: 'none',
          //    backgroundColor: theme.palette.primary.hover
          // },
        },
        sizeLarge: {
          height: 48,
        },
        contained: {
          color: theme.palette.grey[800],
          '&:hover': {
            //boxShadow: 'none',
            backgroundColor: theme.palette.primary.hover,
          },
        },
        containedInherit: {
          color: theme.palette.grey[800],
          boxShadow: theme.customShadows.z8,
          '&:hover': {
            // backgroundColor: theme.palette.primary.hover,
          },
        },
        containedPrimary: {
          boxShadow: theme.customShadows.primary,
        },
        containedSecondary: {
          boxShadow: theme.customShadows.secondary,
        },
        outlinedInherit: {
          border: `1px solid ${theme.palette.grey[500_32]}`,
          // '&:hover': {
          //   backgroundColor: theme.palette.action.hover,
          // },
        },
        textInherit: {
          // '&:hover': {
          //   backgroundColor: theme.palette.action.hover,
          // },
        },
      },
    },
    '&.MuiButton-contained': {
      color: 'yellow',
    },
  }
}
