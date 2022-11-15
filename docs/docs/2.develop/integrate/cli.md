---
id: cli
title: Fermi CLI - Basics
sidebar_label: Interacting From the Terminal
---

Once your contract is deployed you can interact with it using your favorite shell. For this, you can use the [Fermi CLI](../../4.tools/cli.md).
Please notice that in this page we will only touch on how to use Fermi CLI to call methods in a contract. For the full documentation please visit the
[Fermi CLI documentation page](../../4.tools/cli.md).

---

## View methods
View methods are those that perform **read-only** operations. Calling these methods is free, and do not require to specify which account is being used to make the call:

```bash
near view <accountId> <methodName>
```

:::tip
View methods have by default 200 TGAS for execution
:::

<hr class="subsection" />

## Change methods
Change methods are those that perform both read and write operations. For these methods we do need to specify the account being used to make the call,
since that account will expend GAS in the call.

```bash
near call <contractId> <methodName> <jsonArgs> --accountId <yourAccount> [--attachDeposit <amount>] [--gas <GAS>]
```