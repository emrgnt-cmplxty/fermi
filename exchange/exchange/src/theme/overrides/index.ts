import { Theme } from '@mui/material/styles'

import Autocomplete from './Autocomplete'
import Avatar from './Avatar'
import Backdrop from './Backdrop'
import Button from './Button'
import Card from './Card'
import CssBaseline from './CssBaseline'
import Input from './Input'
import LinearProgress from './LinearProgress'
import Paper from './Paper'
import Tooltip from './Tooltip'
import Typography from './Typography'

export default function ComponentsOverrides(theme: Theme) {
  return Object.assign(
    Autocomplete(theme),
    Avatar(),
    Backdrop(theme),
    Button(theme),
    Card(theme),
    CssBaseline(),
    Input(theme),
    LinearProgress(),
    Paper(),
    // Tooltip(theme),
    Typography(theme),
  )
}
