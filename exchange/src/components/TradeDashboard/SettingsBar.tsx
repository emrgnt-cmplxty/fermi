import { Save as SaveIcon } from '@mui/icons-material'
import { Card, IconButton } from '@mui/material'
import { makeStyles } from '@mui/styles'

import AddList from './AddList'

const useStyles = makeStyles((theme) => ({
  root: {
    //padding: theme.spacing(1),
    width: '100%',
    display: 'flex',
    justifyContent: 'flex-end',
  },
}))

interface SettingsBarProps {
  onLayoutSave: () => void
  originalItems: string[]
  items: string[]
  onRemoveItem: (itemId: string) => void
  onAddItem: (itemId: string) => void
}

export default function SettingsBar({
  onLayoutSave,
  originalItems,
  items,
  onRemoveItem,
  onAddItem,
}: SettingsBarProps) {
  const classes = useStyles()
  return (
    <Card className={classes.root}>
      <AddList
        items={items}
        onRemoveItem={onRemoveItem}
        onAddItem={onAddItem}
        originalItems={originalItems}
      />
      <IconButton aria-label="save" onClick={onLayoutSave}>
        <SaveIcon />
      </IconButton>
    </Card>
  )
}
