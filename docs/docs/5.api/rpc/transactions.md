---
id: transactions
title: Transactions
sidebar_label: Transactions
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

The JSON-RPC enables you to send transactions and query their status.

---

## submitTransaction {#submit-transaction}

> Submits a transaction to the network

- method: `submitTransaction`
- params:
  - `[SignedTransaction bytes as hex string]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "submitTransaction",
  "params": {signed_transaction_bytes_in_hex}
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await axionClient.submitTransaction(signedTransaction);
```

</TabItem>
</Tabs>


<details>
<summary>Example response:</summary>
<p>

```json
{
  "jsonrpc": "2.0",
  "result": {
    "Success"
  },
  "id": "axion"
}
```

</p>
</details>

---

## submitTransactionAndConfirm {#submit-transaction-and-confirm}

> Submits a transaction to the network and awaits a reply

- method: `submitTransactionAndConfirm`
- params:
  - `[SignedTransaction bytes as hex string]`


<Tabs>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
  console.log('Building client')
  let client = new TenexClient(DEFAULT_JSONRPC_ADDRESS)

  const receiver = await getPublicKey(exampleData.receiverPrivateKey)
  const paymentRequest = transaction.buildPaymentRequest(receiver, /* assetId */ 0, /* quantity */ 100)
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
```

</TabItem>
</Tabs>


<details>
<summary>Example response:</summary>
<p>

```json
{
  "jsonrpc": "2.0",
  "result": {
    {
      "executed_transaction":  { 
          "signed_transaction": [
            10, 113,  10,   0,  18,  32, 116, 179, 128, 151, 130,   1,
              1, 241, 120, 159,   9, 236,  89,  30, 201, 133, 165, 255,
            190, 150, 231, 118,  49,   6, 103,  14,  78, 146, 134, 184,
            133, 177,  32,   1,  42,  32, 132, 156,  64,  32, 201,  96,
            104, 195, 143, 119,  40, 217, 193,  43,  79, 184, 248, 229,
            56, 241,  81,   3,  80,  72,  94, 180, 246, 157,  83, 111,
            215, 133,  48, 232,   7,  58,  36,  10,  32, 176, 153,  95,
            178,  96, 215,  30, 248, 118,  15, 247, 100,  77, 171, 102,
            228,  47, 237, 203,
            ... 81 more items
          ],
          "events": [],
          "result": { "Err": "AccountLookup" } 
        },
        "transactionId": "0xb2d6265783d3084e804fb58cbafbea7ef24cf5bd7c7e6548b2b726404a436e04"
    }
  },
  "id": "axion"
}
```

</p>
</details>