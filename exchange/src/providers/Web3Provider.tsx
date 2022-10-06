import { SignatureLike } from '@ethersproject/bytes'
import {
  JsonRpcProvider,
  Network,
  TransactionResponse,
  // Web3Provider,
} from '@ethersproject/providers'
import { AbstractConnector } from '@web3-react/abstract-connector'
import { useWeb3React } from '@web3-react/core'
import { TorusConnector } from '@web3-react/torus-connector'
import { WalletConnectConnector } from '@web3-react/walletconnect-connector'
import { WalletLinkConnector } from '@web3-react/walletlink-connector'
import { BigNumber, providers } from 'ethers'
import { Web3Context } from 'hooks/useWeb3Context'
import { ReactElement, useCallback, useEffect, useState } from 'react'
import { hexToAscii } from 'utils/formatters'
import { getNetworkConfig } from 'utils/marketsAndNetworksConfig'
import { getWallet, WalletType } from 'utils/walletOptions'
import { AxionAccount, AxionUtils } from 'tenex-axion-sdk'


type transactionType = {
  value?: string | undefined
  from?: string | undefined
  to?: string | undefined
  nonce?: number | undefined
  gasLimit?: BigNumber | undefined
  gasPrice?: BigNumber | undefined
  data?: string | undefined
}


export type Web3Data = {
  connectWallet: (account: AxionAccount.AxionAccount | undefined) => Promise<void>
  disconnectWallet: () => void
  loading: boolean
  connected: boolean,
  publicAddress: string
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  provider: JsonRpcProvider | undefined
  getTxError: (txHash: string) => Promise<string>
  sendTx: (txData: transactionType) => Promise<TransactionResponse>
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  signTxData: (unsignedData: string) => Promise<SignatureLike>
  error: Error | undefined
}

export const Web3ContextProvider: React.FC<{ children: ReactElement }> = ({
  // eslint-disable-next-line react/prop-types
  children,
}) => {
  const {
    library: provider,
    account,
    activate,
    active,
    error,
    deactivate,
    setError,
  } = useWeb3React<providers.Web3Provider>()

  // const [provider, setProvider] = useState<JsonRpcProvider>();
  const [loading, setLoading] = useState(false)
  const [publicAddress, setCurrentAccount] = useState('')
  const [privateKey, setPrivateKey] = useState(Uint8Array.from([]))
  const [publicKey, setPublicKey] = useState(Uint8Array.from([]))
  const [connected, setConnected] = useState(false)

  const disconnectWallet = useCallback(async () => {
    localStorage.removeItem('axionPrivateKeyHex')
    localStorage.removeItem('axionPublicKeyHex')
    deactivate()
    setCurrentAccount("")
    setPublicKey(Uint8Array.from([]))
    setPrivateKey(Uint8Array.from([]))
    setConnected(false)
    setLoading(false)
}, [])


  // connect to the wallet specified by wallet type
  const connectWallet = useCallback(
    async (account: AxionAccount.AxionAccount | undefined) => {
      setLoading(true)
      try {
        if (account === undefined) {
          account = await AxionAccount.generateAccount()
          localStorage.setItem('axionPrivateKeyHex', AxionUtils.bytesToHex(account.privateKey))
          localStorage.setItem('axionPublicKeyHex', AxionUtils.bytesToHex(account.publicKey))
        }
        setCurrentAccount(account.publicAddress)
        setPublicKey(account.privateKey)
        setPrivateKey(account.publicKey)
        setConnected(true)
        setLoading(false)
      } catch (e) {
        setError(e)
        disconnectWallet();
        setLoading(false)
      }
    },
    [],
  )

    // handle logic to eagerly connect to the injected ethereum provider,
  // if it exists and has granted access already
  useEffect(() => {
    const axionPrivateKeyHex = localStorage.getItem('axionPrivateKeyHex')
    const axionPublicKeyHex = localStorage.getItem('axionPublicKeyHex')
    if (axionPrivateKeyHex && axionPublicKeyHex) {
      const account =  {
        privateKey: AxionUtils.hexToBytes(axionPrivateKeyHex),
        publicKey: AxionUtils.hexToBytes(axionPublicKeyHex),
        publicAddress: axionPublicKeyHex
      } as AxionAccount.AxionAccount
      connectWallet(account).catch((e) => {throw Error(e)})
    }
  }, [])

  // Tx methods

  // TODO: we use from instead of publicAddress because of the mock wallet.
  // If we used current account then the tx could get executed
  const sendTx = async (
    txData: transactionType,
  ): Promise<TransactionResponse> => {
    if (provider) {
      const { from, ...data } = txData
      const signer = provider.getSigner(from)
      const txResponse: TransactionResponse = await signer.sendTransaction({
        ...data,
        value: data.value ? BigNumber.from(data.value) : undefined,
      })
      return txResponse
    }
    throw new Error('Error sending transaction. Provider not found')
  }

  // TODO: recheck that it works on all wallets
  const signTxData = async (unsignedData: string): Promise<SignatureLike> => {
    if (provider && publicAddress) {
      const signature: SignatureLike = await provider.send(
        'eth_signTypedData_v4',
        [publicAddress, unsignedData],
      )

      return signature
    }
    throw new Error('Error initializing permit signature')
  }


  const getTxError = async (txHash: string): Promise<string> => {
    if (provider) {
      const tx = await provider.getTransaction(txHash)
      // @ts-expect-error TODO: need think about "tx" type
      const code = await provider.call(tx, tx.blockNumber)
      const error = hexToAscii(code.substr(138))
      return error
    }
    throw new Error('Error getting transaction. Provider not found')
  }

  return (
    <Web3Context.Provider
      value={{
        web3ProviderData: {
          connectWallet,
          disconnectWallet,
          loading,
          connected,
          publicAddress,
          privateKey,
          publicKey,
          provider,
          getTxError,
          sendTx,
          signTxData,
          error,
        },
      }}
    >
      {children}
    </Web3Context.Provider>
  )
}
