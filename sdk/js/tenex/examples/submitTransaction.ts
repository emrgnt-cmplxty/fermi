// IMPORTS

// INTERNAL
import TenexClient from '../src/tenex/client'

import { transaction, utils } from '../src/tenex'
import { exampleData } from './data'

// EXTERNAL
import { getPublicKey } from '@noble/ed25519'

// UTILITIES
// TODO move towards sandbox implementation - https://github.com/fermiorg/fermi/issues/186
const DEFAULT_JSONRPC_ADDRESS = 'http://localhost:3006'

async function main() {
  console.log('Building client')
  let client = new TenexClient(DEFAULT_JSONRPC_ADDRESS)
  const receiver = await getPublicKey(exampleData.receiverPrivateKey)
  const paymentRequest = transaction.buildPaymentRequest(receiver, 0, 100)
  const signedTransaction = await utils.buildSignedTransaction(
    /* request */ paymentRequest,
    /* senderPrivKey */ exampleData.senderPrivateKey,
    /* recentBlockDigest */ undefined,
    /* fee */ undefined,
    /* client */ client
  )
  console.log('Submitting transaction')
  const result = await client.sendAndConfirmTransaction(signedTransaction)
  console.log('Result: ', result)
  // An example response follows below:
  //
  //
  // Result:  {
  //   executed_transaction: {
  //     signed_transaction: [
  //        10, 113,  10,   0,  18,  32, 116, 179, 128, 151, 130,   1,
  //         1, 241, 120, 159,   9, 236,  89,  30, 201, 133, 165, 255,
  //       190, 150, 231, 118,  49,   6, 103,  14,  78, 146, 134, 184,
  //       133, 177,  32,   1,  42,  32, 132, 156,  64,  32, 201,  96,
  //       104, 195, 143, 119,  40, 217, 193,  43,  79, 184, 248, 229,
  //        56, 241,  81,   3,  80,  72,  94, 180, 246, 157,  83, 111,
  //       215, 133,  48, 232,   7,  58,  36,  10,  32, 176, 153,  95,
  //       178,  96, 215,  30, 248, 118,  15, 247, 100,  77, 171, 102,
  //       228,  47, 237, 203,
  //       ... 81 more items
  //     ],
  //     events: [],
  //     result: { Err: 'AccountLookup' }
  //   },
  //   transaction_id: '0xae8a918eb371777ff07f42e2581f17f64e92af4e0cc492df493ecec9e6b46383'
  // }
  //
  //
  //
}
main()
