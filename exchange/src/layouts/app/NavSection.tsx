import ArrowDropDownIcon from '@mui/icons-material/ArrowDropDown'
import CandlestickChartTwoToneIcon from '@mui/icons-material/CandlestickChartTwoTone'
import Filter3TwoToneIcon from '@mui/icons-material/Filter3TwoTone'
import LocalFireDepartmentTwoToneIcon from '@mui/icons-material/LocalFireDepartmentTwoTone'
import MonetizationOnTwoToneIcon from '@mui/icons-material/MonetizationOnTwoTone'
import {
  Box,
  Grid,
  Link,
  List,
  ListItemButton,
  ListItemButtonProps,
  ListItemIcon,
  ListItemText,
  ListItemTextProps,
  Menu,
  MenuItem,
  MenuItemProps,
  MenuProps,
  Stack,
  Typography,
  useMediaQuery,
} from '@mui/material'
import { styled } from '@mui/material/styles'
import { useTheme } from '@mui/material/styles'
import logo from 'assets/images/logo.png'
import SymbolsInfo from 'components/SymbolsInfo'
import { NavItem } from 'layouts/app/navSectionItems'
import {
  bindHover,
  bindMenu,
  usePopupState,
} from 'material-ui-popup-state/hooks'
import HoverMenu from 'material-ui-popup-state/HoverMenu'
import { ElementType, useState } from 'react'
import {
  matchPath,
  NavLink as RouterLink,
  To,
  useLocation,
} from 'react-router-dom'

import navSectionItems from './navSectionItems'

// TODO
// 1.) change hover menu background
// 2.) remove hacky mt, mb padding fix to hover menu background.

interface ListItemButtonStyledProps extends ListItemButtonProps {
  component?: ElementType
  to?: To
}

export const ListItemButtonStyled = styled((props) => (
  <ListItemButton disableGutters {...props} />
))<ListItemButtonStyledProps>(({ theme }) => ({
  ...theme.typography.body2,
  height: 48,
  position: 'relative',
  textTransform: 'capitalize',
  color: theme.palette.text.primary,
  borderRadius: theme.shape.borderRadius,
  '&:hover': {
    color: theme.palette.primary.hover,
  },
}))

export const ListItemTextStyled = styled((props) => (
  <ListItemText {...props} />
))<ListItemTextProps>(({ theme }) => ({
  color: theme.palette.text.navbar,
}))

export const HoverMenuStyled = styled((props) => (
  <HoverMenu {...props} />
))<MenuProps>(({ theme }) => ({
  '& .MuiPaper-root': {
    backgroundColor: theme.palette.background.navmenu,
  },
}))

export const MenuItemStyled = styled((props) => (
  <MenuItem {...props} />
))<MenuItemProps>(({ theme }) => ({
  '&:hover': {
    color: theme.palette.primary.hover,
  },
}))

interface NavListItemProps {
  item: NavItem
  selected: (path: string) => boolean
}

function NavListItem({ item, selected }: NavListItemProps) {
  const isSelected = selected(item.title)
  const { title, path, icon, info, children } = item

  const popupState = usePopupState({
    variant: 'popover',
    popupId: `${title}-NavListMenu`,
  })

  const selectedStyle = {
    color: 'primary.main',
    fontWeight: 'bold',
  }

  const activeStyle = {
    color: 'primary.main',
  }
  const handleClose = () => {
    popupState.close()
  }
  // const open = overButton || overMenu
  const isDropDown =
    children || item.title === 'spot' || item.title === 'futures'

  return (
    <div>
      <ListItemButtonStyled
        disabled={
          path.includes('governance') ||
          path.includes('more') ||
          path.includes('vault')
        }
        component={title != 'more' ? RouterLink : undefined}
        to={title != 'more' ? path : undefined}
        disableRipple={title === 'more'}
        sx={{
          // embolden & colorize if selected
          ...(isSelected && selectedStyle),
          // colorize if active
          ...(popupState.isOpen && activeStyle),
          pl: 1.5,
          pr: 1.5,
          pb: -1.5,
        }}
        selected={popupState.isOpen}
        {...bindHover(popupState)}
        key={`${title}-NavListMenu-Buttonf`}
      >
        <ListItemTextStyled
          sx={{
            ...(isSelected && selectedStyle),
            ...(popupState.isOpen && activeStyle),
          }}
          disableTypography
          primary={title}
          color="red"
        />
        {isDropDown && (
          <ArrowDropDownIcon
            sx={{
              color: !popupState.isOpen ? 'text.navbar' : 'primary',
              width: 16,
              height: 16,
              ml: 1,
            }}
          />
        )}
      </ListItemButtonStyled>

      {
        // special handling for spot tab
        item.title === 'spot' && (
          <div>
            <HoverMenuStyled {...bindMenu(popupState)} key={'spot'}>
              <SymbolsInfo width={400} type="spot" onClick={handleClose} />
            </HoverMenuStyled>
          </div>
        )
      }
      {
        // process children into dropdown menu
        children && (
          <div>
            <HoverMenuStyled {...bindMenu(popupState)} key={'xx'}>
              {children.map((ele) => {
                return (
                  <MenuItemStyled
                    onClick={handleClose}
                    style={{ width: 350, minWidth: 350 }}
                    sx={{ p: 1 }}
                    key={ele.title}
                    to={ele.path}
                    component={RouterLink}
                    disabled={
                      ele.title === 'Light Mode' || ele.title === '3x Tokens'
                    }
                  >
                    {ele.title === 'Pro Mode' && (
                      <CandlestickChartTwoToneIcon sx={{ mr: 1 }} />
                    )}
                    {ele.title === 'Light Mode' && (
                      <MonetizationOnTwoToneIcon sx={{ mr: 1 }} />
                    )}
                    {ele.title === '3x Tokens' && (
                      <Filter3TwoToneIcon sx={{ mr: 1 }} />
                    )}
                    <Grid container direction="column">
                      <Grid item>
                        <Typography variant="h6" sx={{ pl: 1 }}>
                          {ele.title}
                        </Typography>
                      </Grid>
                      <Grid item sx={{ pl: 1, mt: -2, mb: 1 }}>
                        {ele.subheader && (
                          <Typography variant="h6" color="textSecondary">
                            <br />
                            {ele.subheader}
                          </Typography>
                        )}
                      </Grid>
                    </Grid>
                  </MenuItemStyled>
                )
              })}
            </HoverMenuStyled>
          </div>
        )
      }
    </div>
  )
}

export default function NavSection({ ...rest }) {
  const { pathname } = useLocation()
  const checkSelected = (path: string) => {
    return path && pathname.includes(path)
  }

  const theme = useTheme()
  const matchUpLg = useMediaQuery(theme.breakpoints.up('lg'))

  return (
    matchUpLg && (
      <Box {...rest}>
        <List disablePadding sx={{ p: 2 }} component={Stack} direction="row">
          <Typography
            color="primary"
            sx={{ ml: -4.5, mr: 2, mt: 1.35 }}
            style={{ fontFamily: 'Soehne' }}
            variant="h4"
            component={Link}
            underline="none"
            href={'/'}
          >
            <Grid container direction="row" sx={{ mt: -0.325, mb: 0.325 }}>
              <Grid sx={{ mt: 0.325, mr: -3.5 }}>
                <ListItemIcon>
                  <img style={{ width: '21px', height: '21px' }} src={logo} />
                </ListItemIcon>
              </Grid>
              DMEX
            </Grid>
          </Typography>
          {navSectionItems.map((item, i) => (
            <div key={i}>
              <NavListItem item={item} selected={checkSelected} />
            </div>
          ))}
        </List>
      </Box>
    )
  )
}
