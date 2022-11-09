---
id: futures-markets
title: Futures Markets
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

The JSON-RPC enables you to query the network and get details around deployed futures markets.

---

## getFuturesMarketPlaces {#get-latest-block-info}

> Queries network and returns data around the available futures market places

- method: `getFuturesMarketPlaces`
- params:
  - `[null]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "getFuturesMarketPlaces",
  "params": []
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await tenexClient.getFuturesMarketPlaces();
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
    [
      {
        "quote_asset_id": 1,
        "supported_base_asset_ids": [0],
        "admin": "0x409aa8642d2b75eaa7ebc6fb5413e2abbc30e78bddef59130570d3066b6c3888",
      },
    ]
  },
  "id": "axion"
}
```

</p>
</details>

---


## getFuturesMarkets {#get-latest-block-info}

> Queries network and returns data the available futures markets for a given market place

- method: `getFuturesMarkets`
- params:
  - `[marketAdminPubKey in hex]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "getFuturesMarkets",
  "params": ["0x409aa8642d2b75eaa7ebc6fb5413e2abbc30e78bddef59130570d3066b6c3888"]
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await tenexClient.getFuturesMarketPlaces();
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
    [
      {
        "max_leverage": 20,
        "base_asset_id": 0,
        "quote_asset_id": 1,
        "open_interest": 100,
        "last_traded_price": 1000,
        "oracle_price": 1000000
      }
    ]
  },
  "id": "axion"
}
```

</p>
</details>

---



## getUserMarketplaceInfo {#get-latest-block-info}

> Queries network and returns user data for a given market place

- method: `getUserMarketplaceInfo`
- params:
  - `[marketAdminPubKey in hex, userPubKey in hex]`


<Tabs>
<TabItem value="json" label="JSON" default>

```json
{
  "jsonrpc": "2.0",
  "id": "axion",
  "method": "getUserMarketplaceInfo",
  "params": ["0x409aa8642d2b75eaa7ebc6fb5413e2abbc30e78bddef59130570d3066b6c3888", "0x409aa8642d2b75eaa7ebc6fb5413e2abbc30e78bddef59130570d3066b6c3888"]
}
```

</TabItem>
<TabItem value="🌐 JavaScript" label="JavaScript">

```js
const response = await tenexClient.getUserMarketplaceInfo(marketAdminPubKey, userPubKey);
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
    [
      {
        "user_deposit": 1000000,
        "user_collateral_req": 5095001,
        "user_unrealized_pnl": 99900000,
        "user_market_info": [ { orders: [Array], position: [Object], base_asset_id: 0 } ],
        "quote_asset_id": 1
      }
    ]
  },
  "id": "axion"
}
```

</p>
</details>

---