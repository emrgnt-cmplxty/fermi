import { Button, Grid, useMediaQuery } from '@mui/material'
import { useTheme } from '@mui/material/styles'
import { useState } from 'react'
import { MarketType } from 'utils/globals'
import { useWeb3Context } from 'hooks/useWeb3Context'
import { testnetData } from '../testnet/config'
import { AxionClient, AxionTypes, AxionUtils, AxionAccount, TenexTransaction, TenexUtils } from 'tenex-axion-sdk'

async function airdrop(userPublicKey: AxionAccount.publicKey) {

  let client = new AxionClient(testnetData.defaultJSONRPC)
  let privateKeyHex = testnetData.airdropPrivateKey
  let faucetPrivateKey = AxionUtils.hexToBytes(privateKeyHex)
  let faucetPublicKey = await AxionAccount.getPublicKey(faucetPrivateKey);
  const paymentRequest = TenexTransaction.buildPaymentRequest(faucetPublicKey, 0, 100)

  const signedTransaction = await TenexUtils.buildSignedTransaction(
    /* request */ paymentRequest,
    /* senderPrivKey */ faucetPrivateKey,
    /* recentBlockDigest */ undefined,
    /* fee */ undefined,
    /* client */ client
  )
  const result: AxionTypes.QueriedTransaction = await client.sendAndConfirmTransaction(signedTransaction)

  let status = result.executed_transaction.result;
  if (!status.hasOwnProperty("Ok")) {
    throw Error(result.executed_transaction.result)
  }

  console.log("Successfully airdropped!")
}

function FaucetPage() {
  const theme = useTheme()
  const currentContext = useWeb3Context()
  const matchUpXl = useMediaQuery(theme.breakpoints.up('lg'))
  return (
    <Grid container justifyContent={matchUpXl ? 'center' : 'flex-start'}>
      <Grid sx={{ maxWidth: matchUpXl ? 1500 : undefined }}>
        <Button 
          onClick={async () => {
            await airdrop(currentContext.publicKey);
          }}
        >
          {currentContext.publicAddress != "" ? "Click here to get testnet GRAV and USDC" : "Connect a wallet"}
        </Button>
      </Grid>
    </Grid>
  )
}

export default FaucetPage
