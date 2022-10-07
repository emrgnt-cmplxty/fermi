import {  Box, Typography } from '@mui/material'
import { Variant } from '@mui/material/styles/createTypography'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext } from 'react'

interface DisplayBoxProps {
  title: string
  data: string
  metaText?: string
  primaryVariant?: Variant
  metaVariant?: Variant
}

export const DisplayBox = ({
  title,
  data,
  metaText,
  primaryVariant = 'body1',
  metaVariant = 'body2',
}: DisplayBoxProps) => {
  const {
    settingsState: { theme },
  } = useContext(SettingsContext)
  return (
    <Box display="flex" flexDirection="column" py={3} px={3} width="50%">
      <Box>
        <Typography color="GrayText" variant="h5">
          {title}
        </Typography>
      </Box>
      <Box pt={1}>
        <Typography variant={primaryVariant}>{data}</Typography>
      </Box>
      <Typography variant={metaVariant}>{metaText}</Typography>
    </Box>
  )
}
