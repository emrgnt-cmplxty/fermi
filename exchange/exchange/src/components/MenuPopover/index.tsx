import { Popover, PopoverProps } from '@mui/material'
import { alpha, styled } from '@mui/material/styles'
import { ReactNode } from 'react'

const ArrowStyled = styled('span')(({ theme }) => ({
  [theme.breakpoints.up('sm')]: {
    top: -7,
    zIndex: 1,
    width: 12,
    right: 20,
    height: 12,
    content: "''",
    position: 'absolute',
    borderRadius: '0 0 4px 0',
    transform: 'rotate(-135deg)',
    background: theme.palette.background.paper,
    borderRight: `solid 1px ${alpha(theme.palette.grey[500], 0.12)}`,
    borderBottom: `solid 1px ${alpha(theme.palette.grey[500], 0.12)}`,
  },
}))

interface MenuPopoverProps extends PopoverProps {
  children: ReactNode
}

function MenuPopover({ children, sx, ...rest }: MenuPopoverProps) {
  return (
    <Popover
      anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      transformOrigin={{ vertical: 'top', horizontal: 'right' }}
      PaperProps={{
        sx: {
          p: 1,
          width: 200,
          overflow: 'inherit',
          ...sx,
        },
      }}
      {...rest}
    >
      <ArrowStyled className="arrow" />

      {children}
    </Popover>
  )
}

export default MenuPopover
