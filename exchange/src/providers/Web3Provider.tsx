import { Web3Context } from 'hooks/useWeb3Context'
import { ReactElement, useCallback, useEffect, useState } from 'react'
import { FermiAccount, FermiUtils, FermiTypes } from 'fermi-js-sdk'
import config from '../../../configs/protonet.json';

export type Web3Data = {
  connectWallet: (account: FermiAccount | undefined) => Promise<void>
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
    async (account: FermiAccount.FermiAccount | undefined) => {
      setLoading(true)
      try {
        const authorities = Object.keys(config['authorities']);
        // @ts-ignore
        const deploymentAuthority = config['authorities'][authorities[0]];
        console.log("authorities=", authorities)
        // uncomment this later when we are ready for arbitrary users
        // if (account === undefined) {
        //   account = await FermiAccount.generateAccount()
        //   localStorage.setItem('axionPrivateKeyHex', FermiUtils.bytesToHex(account.privateKey))
        //   localStorage.setItem('axionPublicKeyHex', FermiUtils.bytesToHex(account.publicKey))
        // }
        if (account === undefined) {

        }
        setCurrentAccount(account.publicAddress)
        setPublicKey(account.privateKey)
        setPrivateKey(account.publicKey)
        setConnected(true)
        setLoading(false)
      } catch (e) {
        console.log("err=", e);
        // setError(e)
        disconnectWallet();
        setLoading(false)
      }
    },
    [],
  )

  const setWalletInfo = async (deploymentAuthority: any) => {
    // @ts-ignore
    const deployerPrivateKey = FermiUtils.hexToBytes(
      deploymentAuthority.private_key
    );
    const deployerPublicKey = await FermiAccount.getPublicKey(deployerPrivateKey);
      const account =  {
        privateKey: deployerPrivateKey,
        publicKey: deployerPublicKey,
        publicAddress: FermiUtils.bytesToHex(deployerPublicKey),
      } as FermiAccount.FermiAccount
      connectWallet(account).catch((e) => {throw Error(e)})
      console.log("account=", account)
  }

    // handle logic to eagerly connect to the injected ethereum provider,
  // if it exists and has granted access already
  useEffect(() => {
    const authorities = Object.keys(config['authorities']);
    console.log("authorities=", authorities)
    // @ts-ignore
    const deploymentAuthority = config['authorities'][authorities[0]];
    setWalletInfo(deploymentAuthority);
    // uncomment this later when we are reaady for arbitrary users
    // const axionPrivateKeyHex = localStorage.getItem('axionPrivateKeyHex')
    // const axionPublicKeyHex = localStorage.getItem('axionPublicKeyHex')
    // console.log("axionPrivateKeyHex=", axionPrivateKeyHex)
    // console.log("axionPublicKeyHex=", axionPublicKeyHex)
    
    // if (axionPrivateKeyHex && axionPublicKeyHex) {
    //   const account =  {
    //     privateKey: FermiUtils.hexToBytes(axionPrivateKeyHex),
    //     publicKey: FermiUtils.hexToBytes(axionPublicKeyHex),
    //     publicAddress: axionPublicKeyHex
    //   } as FermiAccount.FermiAccount
    //   connectWallet(account).catch((e) => {throw Error(e)})
    // }
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
