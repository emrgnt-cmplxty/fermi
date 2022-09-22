import { Settings } from '@mui/icons-material'
import {
  Avatar,
  Box,
  Button,
  Divider,
  IconButton,
  MenuItem,
  Stack,
} from '@mui/material'
import MenuPopover from 'components/MenuPopover'
import { useRef, useState } from 'react'

interface SettingsPopoverItem {
  title: string
  icon: JSX.Element
  onClick?: () => void
}

interface SettingsPopoverProps {
  items: SettingsPopoverItem[]
}

function SettingsPopover({ items }: SettingsPopoverProps) {
  const anchorRef = useRef(null)

  const [open, setOpen] = useState(null)

  // TODO: not any
  const handleOpen = (event: any) => {
    setOpen(event.currentTarget)
  }

  const handleClose = () => {
    setOpen(null)
  }

  return (
    <>
      <IconButton
        ref={anchorRef}
        onClick={handleOpen}
        sx={{ width: 40, height: 40 }}
      >
        <Settings />
      </IconButton>

      <MenuPopover
        open={!!open}
        anchorEl={open}
        onClose={handleClose}
        sx={{
          p: 0,
          mt: 1.5,
          ml: 0.75,
          '& .MuiMenuItem-root': {
            typography: 'body2',
            borderRadius: 0.75,
          },
        }}
      >
        <Box sx={{ my: 1.5, px: 2.5 }}>
          {' '}
          <Stack
            alignItems="center"
            spacing={3}
            sx={{ borderRadius: 2, position: 'relative' }}
          >
            <Button href="#" variant="contained">
              Connect Wallet
            </Button>
          </Stack>
        </Box>

        <Divider sx={{ borderStyle: 'dashed' }} />

        <Stack sx={{ p: 1 }}>
          {items.map((item) => (
            <MenuItem key={item.title} onClick={item.onClick}>
              {item.icon}
              {item.title}
            </MenuItem>
          ))}
        </Stack>
      </MenuPopover>
    </>
  )
}

export default SettingsPopover
