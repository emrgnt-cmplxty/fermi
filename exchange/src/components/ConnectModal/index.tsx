import {
  Alert,
  Box,
  Button,
  Fade,
  Grid as Trans,
  Link,
  Modal,
  Typography,
} from '@mui/material'
import browserWallet from 'assets/icons/wallets/browserWallet.svg'
import coinbase from 'assets/icons/wallets/coinbase.svg'
import frame from 'assets/icons/wallets/frame.svg'
import torus from 'assets/icons/wallets/torus.svg'
import walletConnect from 'assets/icons/wallets/walletConnect.svg'
import KeyIcon from '@mui/icons-material/Key';
import { useWeb3Context } from 'hooks/useWeb3Context'
import styles from './index.module.scss'

export enum WalletType {
  INJECTED = 'injected',
  WALLET_CONNECT = 'wallet_connect',
  WALLET_LINK = 'wallet_link',
  TORUS = 'torus',
  FRAME = 'frame',
  GNOSIS = 'gnosis',
  IN_MEMORY = 'in_memory',
}


export type WalletRowProps = {
  walletName: string
  walletType: WalletType
}

const WalletRow = ({ walletName, walletType }: WalletRowProps) => {
  const { connectWallet } = useWeb3Context()

  const getWalletIcon = (walletType: WalletType) => {
    switch (walletType) {
      case WalletType.INJECTED:
        return (
          <img
            src={browserWallet}
            width="24px"
            height="24px"
            alt={`browser wallet icon`}
          />
        )
      case WalletType.WALLET_CONNECT:
        return (
          <img
            src={walletConnect}
            width="24px"
            height="24px"
            alt={`browser wallet icon`}
          />
        )
      case WalletType.WALLET_LINK:
        return (
          <img
            src={coinbase}
            width="24px"
            height="24px"
            alt={`browser wallet icon`}
          />
        )
      case WalletType.TORUS:
        return (
          <img
            src={torus}
            width="24px"
            height="24px"
            alt={`browser wallet icon`}
          />
        )
      case WalletType.FRAME:
        return (
          <img
            src={frame}
            width="24px"
            height="24px"
            alt={`browser wallet icon`}
          />
        )
      case WalletType.IN_MEMORY:
          return (
            <KeyIcon />
          )
      default:
        return null
    }
  }

  return (
    <Button
      variant="outlined"
      sx={{
        display: 'flex',
        flexDirection: 'row',
        justifyContent: 'space-between',
        width: '100%',
        mb: '8px',
      }}
      size="large"
      onClick={() => connectWallet(undefined)}
      endIcon={getWalletIcon(walletType)}
      disabled={walletType != WalletType.IN_MEMORY}
    >
      {walletName}
    </Button>
  )
}

export enum ErrorType {
  UNSUPORTED_CHAIN,
  USER_REJECTED_REQUEST,
  UNDETERMINED_ERROR,
  NO_WALLET_DETECTED,
}
interface WalletSelectorProps {
  setOpenModal: (newState: boolean) => void
}
const WalletSelector = ({ setOpenModal }: WalletSelectorProps) => {
  const { error } = useWeb3Context()

  let blockingError: ErrorType | undefined = undefined
  if (error) {
      blockingError = ErrorType.UNDETERMINED_ERROR
  }

  const handleBlocking = () => {
      console.log('Uncaught error: ', blockingError)
      return <Trans>Error connecting. Try refreshing the page.</Trans>
  }

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: 'background.navbar',
        maxWidth: 425,
        p: 2,
      }}
    >
      {/* <TxModalTitle title="Connect a wallet" /> */}
      <Box sx={{ pb: 1 }}>Connect a wallet</Box>
      {error && (
        <Alert severity="error" sx={{ mb: '24px' }}>
          {handleBlocking()}
        </Alert>
      )}
      <WalletRow
        key="browser_wallet"
        walletName="Browser wallet"
        walletType={WalletType.INJECTED}
      />
      <WalletRow
        key="walletconnect_wallet"
        walletName="WalletConnect"
        walletType={WalletType.WALLET_CONNECT}
      />
      <WalletRow
        key="walletlink_wallet"
        walletName="Coinbase"
        walletType={WalletType.WALLET_LINK}
      />
      <WalletRow
        key="torus_wallet"
        walletName="Torus"
        walletType={WalletType.TORUS}
      />
      {/* <WalletRow
        key="frame_wallet"
        walletName="Frame"
        walletType={WalletType.FRAME}
      /> */}
      <WalletRow
        key="in_memory_wallet"
        walletName="In Browser Memory"
        walletType={WalletType.IN_MEMORY}
      />
      <Typography
        variant="helper"
        sx={{ mt: '22px', mb: '30px', alignSelf: 'center' }}
      >
        <Trans>
          Need help connecting a wallet?{' '}
          <Link href="https://docs.dmex.fi/faq/troubleshooting" target="_blank">
            Read our FAQ
          </Link>
        </Trans>
      </Typography>
      <Typography variant="caption">
        <Trans>
          Wallets are provided by External Providers and by selecting you agree
          to Terms of those Providers. Your access to the wallet might be
          reliant on the External Provider being operational.
        </Trans>
      </Typography>
    </Box>
  )
}

interface LoginModalProps {
  openModal: boolean
  setOpenModal: Dispatch<SetStateAction<boolean>>
}

export default function LoginModal({
  openModal,
  setOpenModal,
}: LoginModalProps) {
  const { publicAddress } = useWeb3Context()

  return (
    <Modal
      aria-labelledby="transition-modal-title"
      aria-describedby="transition-modal-description"
      open={openModal && !publicAddress}
      onClose={(event, reason) => {
        setOpenModal(false)
      }}
      className={styles.modal}
      closeAfterTransition
    >
      <Fade>{openModal && <WalletSelector setOpenModal={setOpenModal} />}</Fade>
    </Modal>
  )
}
