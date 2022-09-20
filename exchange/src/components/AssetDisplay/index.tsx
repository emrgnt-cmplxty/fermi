import ArrowDropDownIcon from '@mui/icons-material/ArrowDropDown'
import { Avatar, Box, Typography } from '@mui/material'
import { Variant } from '@mui/material/styles/createTypography'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext } from 'react'
import { BaseSymbol } from 'utils/globals'
import { SYMBOL_TO_IMAGE_DICT } from 'utils/tokenData'

import styles from './index.module.scss'

interface AssetDisplayProps {
  symbol: BaseSymbol
  leftLabel?: string
  rightLabel?: string
  primaryVariant?: Variant
  rightMetaLabel?: string
  rightMetaVariant?: Variant
  farRightLabel?: string
  farRightVariant?: Variant
  farRightColor?: 'textPrimary' | 'textSecondary'
  farRightPad?: number
  padding?: string | number
  avatarSize?: number
  isActive?: boolean
  isDropDown?: boolean
}

export const AssetDisplay = ({
  symbol,
  leftLabel,
  rightLabel,
  primaryVariant = 'body2',
  rightMetaLabel,
  rightMetaVariant = 'caption',
  farRightLabel,
  farRightVariant = 'caption',
  farRightColor = 'textSecondary',
  farRightPad = 0.5,
  padding = 1,
  avatarSize = 25,
  isActive = false,
  isDropDown = false,
}: AssetDisplayProps) => {
  const {
    settingsState: { theme },
  } = useContext(SettingsContext)
  return (
    <Box display="inline-flex" alignItems="center">
      {leftLabel && (
        <Typography
          display="inline"
          variant={primaryVariant}
          color={isActive ? 'primary' : undefined}
        >
          {leftLabel}
        </Typography>
      )}
      <Avatar
        sx={{ width: avatarSize, height: avatarSize }}
        src={
          SYMBOL_TO_IMAGE_DICT[
            (symbol || '').replace('-PERP', '').replace('-USD', '')
          ]?.light
        }
        alt={symbol}
        className={styles.avatar}
      />
      <Box
        display="flex"
        flexDirection="column"
        justifyContent="center"
        alignContent="center"
        alignItems="flex-start"
        pl={padding}
      >
        <Box display="flex" flexDirection="row" alignItems="center">
          {rightLabel && (
            <Typography
              lineHeight={1}
              pl={0}
              display="inline"
              variant={primaryVariant}
              color={isActive ? 'primary' : undefined}
            >
              {rightLabel}
            </Typography>
          )}
          {farRightLabel && (
            <Typography
              className={styles.farRightLabel}
              lineHeight={1}
              display="inline"
              variant={farRightVariant}
              color={farRightColor}
              sx={{ pl: farRightPad }}
            >
              {farRightLabel}
            </Typography>
          )}
          {isDropDown && (
            <ArrowDropDownIcon color={isActive ? 'primary' : undefined} />
          )}
        </Box>
        {rightMetaLabel && (
          <Typography
            color="textSecondary"
            className={styles.meta}
            variant={rightMetaVariant}
          >
            {rightMetaLabel}
          </Typography>
        )}
      </Box>
    </Box>
  )
}
