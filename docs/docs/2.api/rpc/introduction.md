---
id: introduction
sidebar_label: Home
title: Axion JSON-RPC
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

The JSNO-RPC allows clients to communicate directly with the Axion network. For example,
tools such as [axion-js-sdk](https://github.com/gdexorg/gdex/tree/main/sdk/js/tenex) just create abstractions around the JSON-RPC calls.

<hr class="subsection" />

## RPC Providers

There are multiple [RPC providers which you can choose from](./providers.md). These providers will work as intermediaries to help you interact with the Axion network.

<hr class="subsection" />

## Axion RPC - Quick Links

| API                                        | Description                                                                  |
| ------------------------------------------ | ---------------------------------------------------------------------------- |
| [Block](/api/rpc/block-info)               | Query the network and get details about specific blocks.                     |
| [Transactions](/api/rpc/transactions)      | Submit transactions to the network and await confirmation.                   |

:::tip
You can access the JSON RPC 2.0 endpoints using [Postman](/api/rpc/setup#postman-setup),
[JavaScript](/api/rpc/setup#javascript-setup).
:::

<hr class="subsection" />
