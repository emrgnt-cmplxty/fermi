// IMPORTS
import { test, expect } from '@jest/globals'
import { getPublicKey } from '@noble/ed25519'
import { buildPaymentRequest } from '../utils/bankController'
import {
  buildAccountDepositRequest,
  buildAccountWithdrawalRequest,
  buildFuturesLimitOrderRequest,
} from '../utils/futuresController'
import { transactionParams, getTransactionDigest } from '../utils'
import { testData } from './config'
import { TenexClient } from '../../dist'

// TODO - extend test workflow to cover all transaction types and cleanup the online tests
// https://github.com/gdexorg/gdex/issues/184

test('Payment transaction workflow', async () => {
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)
  const receiver = await getPublicKey(testData.defaultReceiver)
  const sender = await getPublicKey(testData.defaultSender)

  // TRANSACTION
  const paymentRequest = buildPaymentRequest(receiver, 0, 100)

  const transaction = await client.buildTransaction(
    /* request */ paymentRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(getTransactionDigest(transaction)).toStrictEqual(testData.expectedPaymentDigest)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ paymentRequest,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedPaymentSignature)
})

test('Futures deposit workflow', async () => {
  const admin = await getPublicKey(testData.defaultAdmin)
  const sender = await getPublicKey(testData.defaultSender)
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTION
  const depositRequest = buildAccountDepositRequest(1_000, admin)

  const transaction = await client.buildTransaction(
    /* request */ depositRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(getTransactionDigest(transaction)).toStrictEqual(testData.expectedAccountDepositDigest)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ depositRequest,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedAccountDepositSignature)
})

test('Futures withdrawal workflow', async () => {
  const admin = await getPublicKey(testData.defaultAdmin)
  const sender = await getPublicKey(testData.defaultSender)
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTIONS
  const withdrawalRequest = buildAccountWithdrawalRequest(1_000, admin)

  const transaction = await client.buildTransaction(
    /* request */ withdrawalRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(getTransactionDigest(transaction)).toStrictEqual(testData.expectedAccountWithdrawalDigest)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ withdrawalRequest,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedAccountWithdrawalSignature)
})

test('Futures limit order workflow', async () => {
  const admin = await getPublicKey(testData.defaultAdmin)
  const sender = await getPublicKey(testData.defaultSender)
  const client = new TenexClient(testData.defaultRelayer, testData.defaultTransactionSubmitter)

  // TRANSACTIONS
  const orderRequest = buildFuturesLimitOrderRequest(0, 1, transactionParams.orderSide.Ask, 1_000, 1_000, admin)

  const transaction = await client.buildTransaction(
    /* request */ orderRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(getTransactionDigest(transaction)).toStrictEqual(testData.expectedFuturesLimitOrderDigest)

  const signedTransaction = await client.buildSignedTransaction(
    /* request */ orderRequest,
    /* senderPrivKey */ testData.defaultSender,
    /* defaultBlockDigest */ testData.defaultBlockDigest
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedFuturesLimitOrderSignature)
})
