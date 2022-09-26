// IMPORTS
import { test } from '@jest/globals'
import { getPublicKey } from '@noble/ed25519'
import { buildPaymentRequest } from '../utils/bankController'
import {
  buildAccountDepositRequest,
  buildAccountWithdrawalRequest,
  buildCreateMarketplaceRequest,
  buildCreateMarketRequest,
  buildUpdateMarketParamsRequest,
  buildUpdateTimeRequest,
  buildUpdatePricesRequest,
  buildFuturesLimitOrderRequest,
} from '../utils/futuresController'
import { transactionParams } from '../utils'
import { testData } from './config'
import { TenexClient } from '../../dist'

test('Payment transaction workflow', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)
  const receiver = await getPublicKey(testData.defaultReceiver)

  // TRANSACTION
  const request = buildPaymentRequest(receiver, 0, 100)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Build marketplace', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const request = buildCreateMarketplaceRequest(testData.defaultQuoteAssetId)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultAdmin,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Build market', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const request = buildCreateMarketRequest(testData.defaultQuoteAssetId)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultAdmin,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Update market params request', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const request = buildUpdateMarketParamsRequest(testData.defaultQuoteAssetId, 25)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultAdmin,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Update time request', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const request = buildUpdateTimeRequest(1_000_000)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultAdmin,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Update prices request', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const request = buildUpdatePricesRequest([1_000_000])

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultAdmin,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Futures deposit workflow', async () => {
  const admin = await getPublicKey(testData.defaultAdmin)
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const request = buildAccountDepositRequest(1_000, admin)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Futures limit order workflow', async () => {
  const admin = await getPublicKey(testData.defaultAdmin)
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTIONS
  const request = buildFuturesLimitOrderRequest(0, 1, transactionParams.orderSide.Ask, 1_000, 1_000, admin)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})

test('Futures withdrawal workflow', async () => {
  const admin = await getPublicKey(testData.defaultAdmin)
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTIONS
  const request = buildAccountWithdrawalRequest(1_000, admin)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ request,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  const _transactionId = await client.sendAndConfirmTransaction(signedTransaction)
})
