---
id: block-info
title: Block
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

The JSON-RPC enables you to query the network and get details about specific blocks.

---

## getLatestBlockInfo {#get-latest-block-info}

> Queries network and returns meta data around the latest confirmed block.

- method: `getLatestBlockInfo`
- params:
  - `[null]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "getLatestBlockInfo",
  "params": []
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await axionClient.getLatestBlockInfo();
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
    "header": {
      "validator_system_epoch_time_in_micros": 1664914635824250,
      "block_number": 131168,
      "block_id": "0x23a4001a11c639a4130363935e1064d13d3f777f6a2f3496113bf1f5d7bece44",
    },
  },
  "id": "axion"
}
```

</p>
</details>

---

## getBlockInfo {#get-block-info}

> Queries network and returns meta data around the specified block.

- method: `getBlockInfo`
- params: 
  - `[block_number]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "getBlockInfo",
  "params": [1]
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await axionClient.getBlockInfo(1);
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
    "header": {
      "validator_system_epoch_time_in_micros": 1664911300461308,
      "block_number": 1,
      "block_id": "0x7e7366d15e8d91b3ba3928cef72568f0f31751ec600bc30ada81d39061822af6",
    },
  },
  "id": "axion"
}
```

</p>
</details>

---

## getBlock {#get-block}

> Queries network and returns the full transaction data around a specified block.

- method: `getBlock`
- params: 
  - `[block_number]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "getBlock",
  "params": [1]
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await axionClient.getBlock(1);
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
    "transactions": [
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
      },
      ...
    ],
  },
  "id": "axion"
}
```

</p>
</details>
