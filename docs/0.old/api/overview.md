---
id: overview
title: Axion APIs
sidebar_label: Overview
---

Take a look at the different available APIs to help you build amazing apps on Axion.

## JSON-RPC {#rpc-api}

[Axion JSON-RPC](/docs/api/rpc) provides a simple JSON RPC 2.0 API to interact with the Axion blockchain.

| API | Description |
|-----|-------------|
| [Block](/docs/api/rpc/block-info) | Query the network and get details about specific blocks or chunks. |
| [Gas](/docs/api/rpc/gas) | Get gas price for a specific block or hash. |
| [Protocol](/docs/api/rpc/protocol) | Retrieve current genesis and protocol configuration. |
| [Network](/docs/api/rpc/network) | Return status information for nodes and validators. |
| [Transactions](/docs/api/rpc/transactions) | Send transactions and query their status. |
| [Sandbox](/docs/api/rpc/sandbox) | Patch state on a local sandbox node. |

> **Tip:** You can access the JSON RPC 2.0 endpoints using [Postman](/docs/api/rpc#postman-setup),
> [JavaScript](/docs/api/rpc#javascript-setup), and [HTTPie](/docs/api/rpc#httpie-setup).

## REST Server {#rest-server}

[Axion REST API Server](/docs/api/rest-server/overview) is a project that allows you create your own simple
REST API server that interacts with the Axion blockchain.

| Route                                      | Description                                                                                                                 |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| [CONTRACTS](/docs/api/rest-server/contracts)                              |  Deploy, view, and call smart contracts on Axion.         |
| [UTILS](/docs/api/rest-server/utils)                                  |    Init accounts, create sub-accounts, and view key pairs.                                                 |
| [NFTs](/docs/api/rest-server/nfts)                            |        Mint, view, and transfer NFTs.                                       |
