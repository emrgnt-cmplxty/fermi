import { Button, Grid, useMediaQuery } from '@mui/material'
import { useTheme } from '@mui/material/styles'
import { useWeb3Context } from 'hooks/useWeb3Context'
import config from "../../../configs/protonet.json"

import { FermiClient, FermiTypes, FermiUtils, FermiAccount } from 'fermi-js-sdk'
import { TenexTransaction, TenexUtils } from 'tenex-js-sdk'

// TODO - type Authority if we keep a workflow like this
function getJsonRpcUrl(authority: any) {
  // construct the URL dynamically from the multiaddr
  var link = authority['jsonrpc_address'].split('/')[2];
  var port = authority['jsonrpc_address'].split('/')[4];
  var url = "http://" + link + ":" + port;
  return url;
}

async function airdrop(userPublicKey: FermiTypes.PublicKey) {
  const authorities = Object.keys(config["authorities"])
  //@ts-ignore
  const authority = config['authorities'][authorities[0]]
  console.log("authority = ", authority)
  console.log("getJsonRpcUrl(authority) = ", getJsonRpcUrl(authority))
  
  let client = new FermiClient(getJsonRpcUrl(authority))

  let privateKeyHex = authority.private_key
  let faucetPrivateKey = FermiUtils.hexToBytes(privateKeyHex)
  
  const paymentRequest = TenexTransaction.buildPaymentRequest(userPublicKey, 0, 100)
  const signedTransaction = await TenexUtils.buildSignedTransaction(
    /* request */ paymentRequest,
    /* senderPrivKey */ faucetPrivateKey,
    /* recentBlockDigest */ undefined,
    /* fee */ undefined,
    /* client */ client
  )

  console.log("Submitting airdrop now")
  const result: FermiTypes.QueriedTransaction = await client.sendAndConfirmTransaction(signedTransaction)
  console.log("result=", result)

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
