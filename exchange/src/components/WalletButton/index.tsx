import ArrowDropDownIcon from '@mui/icons-material/ArrowDropDown'
import ArrowDropUpIcon from '@mui/icons-material/ArrowDropUp'
import ContentCopyIcon from '@mui/icons-material/ContentCopy'
import LaunchIcon from '@mui/icons-material/Launch'
import LogoutIcon from '@mui/icons-material/Logout'
import {
  Box,
  Button,
  Grid,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Typography,
} from '@mui/material'
// import { useLingui } from '@lingui/react';
import { styled } from '@mui/material/styles'
import LoginModal from 'components/ConnectModal'
import { useWeb3Context } from 'hooks/useWeb3Context'
import { useState } from 'react'
import { maskHash } from 'utils/formatters'

const dropdownItems = [
  { name: 'Copy Address', symbol: 'copy' },
  { name: 'View on Explorer', symbol: 'explorer' },
  { name: 'Disconnect Wallet', symbol: 'logout' },
]

// TODO //
// knit recommends not using sx as in line 21 below.  What is best practice?

export interface ConnectButonProps {
  setOpenLogin: React.Dispatch<React.SetStateAction<boolean>>
}

const ConnectButton = ({ setOpenLogin }: ConnectButonProps) => {
  return (
    <Button
      onClick={setOpenLogin}
      variant={'contained'}
      sx={{
        height: 32,
        '&:hover': {
          //backgroundColor: 'green'
        },
      }}
    >
      <Typography color="background.component" variant="h6">
        Connect
      </Typography>
    </Button>
  )
}

export interface ConnectedButonProps {
  handleConnectedClick: (
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>,
  ) => void
  account: string
  openMenu: string
}

const ConnectedButton = ({
  handleConnectedClick,
  account,
  openMenu,
}: ConnectedButonProps) => {
  return (
    <Button
      variant="outlined"
      onClick={handleConnectedClick}
      sx={{ minWidth: 140 }}
    >
      <Grid container>
        <Grid item>
          <Typography color="text.navbar" variant="h6">
            {maskHash(account)}
          </Typography>
        </Grid>
        <Grid item sx={{ mb: -1, ml: 1, mr: -1 }}>
          {!openMenu && <ArrowDropDownIcon sx={{ color: 'text.navbar' }} />}
          {openMenu && <ArrowDropUpIcon />}
        </Grid>
      </Grid>
    </Button>
  )
}

// export interface WalletButtonProps {}
export const WalletButton = () => {
  const [openLogin, setOpenLogin] = useState(false)
  const [openMenu, setOpenMenu] = useState(false)
  const [anchorEl, setAnchorEl] = useState<Element | null>(null)
  const currentContext = useWeb3Context()

  const handleConnectedClick = (
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>,
  ) => {
    setAnchorEl(event.currentTarget)
    setOpenMenu(true)
  }
  const handleClose = (
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>,
  ) => {
    setAnchorEl(event.currentTarget)
    setOpenMenu(false)
  }

  // const { i18n } = useLingui();

  const clickMenu = (symbol: string) => {
    if (symbol === 'copy') {
      navigator.clipboard.writeText(currentContext.currentAccount)
    }
    if (symbol === 'explorer') {
      window.open(
        `https://etherscan.io/address/${currentContext.currentAccount}`,
        '_blank',
      )
    }
    if (symbol === 'logout') {
      localStorage.removeItem('mockWalletAddress')
      currentContext.disconnectWallet()
    }
    setOpenMenu(false)
  }
  return (
    <Box>
      <LoginModal openModal={openLogin} setOpenModal={setOpenLogin} />
      <Menu
        id="more-menu"
        MenuListProps={{
          'aria-labelledby': 'more-button',
        }}
        anchorEl={anchorEl}
        open={openMenu}
        onClose={handleClose}
        keepMounted={true}
      >
        {dropdownItems.map((item, index) => (
          <MenuItem
            key={index}
            sx={{ width: 250 }}
            onClick={() => {
              clickMenu(item.symbol)
            }}
          >
            <ListItemIcon>
              {item.symbol === 'copy' && (
                <ContentCopyIcon
                  sx={{ fontSize: '20px', color: 'text.primary' }}
                />
              )}
              {item.symbol === 'explorer' && (
                <LaunchIcon sx={{ fontSize: '20px', color: 'text.primary' }} />
              )}
              {item.symbol === 'logout' && (
                <LogoutIcon sx={{ fontSize: '20px', color: 'text.primary' }} />
              )}
            </ListItemIcon>
            <ListItemText>
              <Typography variant="h6">{item.name}</Typography>
            </ListItemText>
          </MenuItem>
        ))}
      </Menu>
      {!currentContext?.connected && (
        <ConnectButton setOpenLogin={setOpenLogin} />
      )}
      {currentContext?.connected && (
        <ConnectedButton
          handleConnectedClick={handleConnectedClick}
          account={currentContext.currentAccount}
          openMenu={openMenu}
        />
      )}
    </Box>
  )
}
