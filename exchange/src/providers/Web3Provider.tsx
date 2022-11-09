import { Web3Context } from 'hooks/useWeb3Context'
import { ReactElement, useCallback, useEffect, useState } from 'react'
import { AxionAccount, AxionUtils, AxionTypes } from 'fermi-js-sdk'




export type Web3Data = {
  connectWallet: (account: AxionAccount | undefined) => Promise<void>
  disconnectWallet: () => void
  loading: boolean
  connected: boolean,
  publicAddress: string
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  getTxError: (txHash: string) => Promise<string>
  sendTx: (txData: any) => Promise<any>
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  signTxData: (unsignedData: string) => Promise<any>
}

export const Web3ContextProvider: React.FC<{ children: ReactElement }> = ({
  // eslint-disable-next-line react/prop-types
  children,
}) => {

  // const [provider, setProvider] = useState<JsonRpcProvider>();
  const [loading, setLoading] = useState(false)
  const [publicAddress, setCurrentAccount] = useState('')
  const [privateKey, setPrivateKey] = useState(Uint8Array.from([]))
  const [publicKey, setPublicKey] = useState(Uint8Array.from([]))
  const [connected, setConnected] = useState(false)

  const disconnectWallet = useCallback(async () => {
    localStorage.removeItem('axionPrivateKeyHex')
    localStorage.removeItem('axionPublicKeyHex')
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
    txData: any,
  ): Promise<any> => {
    throw new Error('Not yet implemented')
  }

  // TODO: recheck that it works on all wallets
  const signTxData = async (unsignedData: string): Promise<any> => {
    throw new Error('Not yet implemented')
  }


  const getTxError = async (txHash: string): Promise<string> => {
    throw new Error('Not yet implemented')
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
          getTxError,
          sendTx,
          signTxData,
        },
      }}
    >
      {children}
    </Web3Context.Provider>
  )
}
