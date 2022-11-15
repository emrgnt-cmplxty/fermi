---
id: whatisacontract
title: What is a Smart Contract?
sidebar_label: What is a Smart Contract?
---

Smart contracts are simple programs that live in a Fermi network. As any modern application, smart contracts store data and expose methods to interact with them.

They are written in human-readable languages, then compiled and deployed to an account where everyone can interact with them.

Do not worry if you don't know how smart-contract blockchains work. As a developer, it is sufficient to understand that Fermi smart-contracts:
1. Have **limited** computational resources.
2. Interact with other contracts in an **asynchronous** way.
3. Deal with **real money**, for which security must be a top concern.

:::info HTTP Requests and Smart Contracts
Smart contracts **cannot perform HTTP requests**, meaning they can't retrieve data from outside the Fermi network. However, they can receive data from any outside source. If needed, you can set up a server to regularly feed them data (this is in short how [Oracles](../relevant-contracts/oracles.md) work).
:::

---

## Programming Languages
Developers can choose between using [Javascript](../../4.tools/js-sdk.md) or [Rust](../../4.tools/js-sdk.md) to write smart contracts in Fermi.

Indistinctly from the language chosen, the Fermi SDK will help you to compile the contract into WebAssembly, from which point it can be deployed and executed on the Fermi platform.


While it is not necessary to be an expert in either language, during these docs we will assume you have a basic knowledge of at least one of them.

### Other Languages
Theoretically, you can use any language that compiles to Wasm for developing Fermi smart contract. However, in order to have a user-friendly experience we would need
to provide a library that wraps around low-level runtime APIs, while also offering other high-level functionalities.

We envision that in the future, more languages will be supported and the support will be done through the effort from the wider community, not just Fermi alone.