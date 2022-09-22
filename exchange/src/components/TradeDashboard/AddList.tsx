import { AddCircleOutline as AddCircleOutlineIcon } from '@mui/icons-material'
import {
  Checkbox,
  FormControl,
  FormControlLabel,
  FormGroup,
  FormLabel,
  Grid,
  IconButton,
  Popover,
} from '@mui/material'
import { makeStyles } from '@mui/styles'
import React from 'react'

const useStyles = makeStyles((theme) => ({
  popup: {
    padding: theme.spacing(2),
  },
}))

interface AddListProps {
  originalItems: string[]
  items: string[]
  onRemoveItem: (itemId: string) => void
  onAddItem: (itemId: string) => void
}

const widgetNames = {
  a: 'A',
  b: 'B',
  c: 'C',
  d: 'D',
  e: 'E',
}

export default function AddList({
  originalItems,
  items,
  onRemoveItem,
  onAddItem,
}: AddListProps) {
  const classes = useStyles()
  const [anchorEl, setAnchorEl] = React.useState(null)

  const handleClick = (event) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const open = Boolean(anchorEl)
  const id = open ? 'simple-popover' : undefined

  const handleChange = (e) => {
    if (e.target.checked) {
      onAddItem(e.target.name)
    } else {
      onRemoveItem(e.target.name)
    }
  }

  return (
    <>
      <IconButton aria-label="add" onClick={handleClick} aria-describedby={id}>
        <AddCircleOutlineIcon />
      </IconButton>
      <Popover
        id={id}
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'center',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'center',
        }}
      >
        <Grid className={classes.popup}>
          <FormControl component="fieldset">
            <FormLabel component="legend">Select Widgets</FormLabel>
            <FormGroup>
              {originalItems.map((i) => (
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={items.includes(i)}
                      onChange={handleChange}
                      name={i}
                    />
                  }
                  label={widgetNames[i]}
                  key={i}
                />
              ))}
            </FormGroup>
          </FormControl>
        </Grid>
      </Popover>
    </>
  )
}
