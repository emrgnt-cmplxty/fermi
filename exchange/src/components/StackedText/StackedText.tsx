import { Avatar, Divider, Typography } from '@mui/material'
import { Variant } from '@mui/material/styles/createTypography'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext } from 'react'

import styles from './StackedText.module.scss'

interface StackedTextProps {
  topText: any
  bottomText: any
  topTextVariant?: Variant
  bottomTextVariant?: Variant
  topTextColor?: 'textPrimary' | 'textSecondary'
  bottomTextColor?: 'textPrimary' | 'textSecondary'
  textAlign?: string
  showDivider?: boolean
}

export const StackedText = ({
  topText,
  bottomText,
  topTextVariant = 'body2',
  bottomTextVariant = 'body2',
  topTextColor = 'textPrimary',
  bottomTextColor = 'textSecondary',
  textAlign = 'right',
  showDivider = false,
}: StackedTextProps) => {
  const variableStyles = { '--text-align': textAlign } as React.CSSProperties
  return (
    <div className={styles.container} style={variableStyles}>
      <Typography
        className={styles.top}
        display="inline"
        variant={topTextVariant}
        color={topTextColor}
        component="div"
      >
        {topText}
      </Typography>

      {showDivider && <Divider className={styles.divider} />}

      <Typography
        className={styles.bottom}
        variant={bottomTextVariant}
        color={bottomTextColor}
        component="div"
      >
        {bottomText}
      </Typography>
    </div>
  )
}
