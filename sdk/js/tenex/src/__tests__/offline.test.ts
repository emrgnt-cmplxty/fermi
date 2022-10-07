// IMPORTS

// INTERNAL
import {
  buildAccountDepositRequest,
  buildAccountWithdrawalRequest,
  buildFuturesLimitOrderRequest,
  buildPaymentRequest,
} from '../tenex/transaction'
import { transactionParams, buildTransaction, buildSignedTransaction } from '../tenex/utils'
import { testData } from './config'
import { AxionUtils } from 'axion-js-sdk'

// EXTERNAL
import { test, expect } from '@jest/globals'
import { getPublicKey } from '@noble/ed25519'

// TODO - extend test workflow to cover all transaction types and cleanup the online tests
// https://github.com/gdexorg/gdex/issues/184

test('Payment transaction workflow', async () => {
  const receiver = await getPublicKey(testData.receiverPrivateKey)
  const sender = await getPublicKey(testData.senderPrivateKey)

  // // TRANSACTION
  const paymentRequest = buildPaymentRequest(receiver, 0, 100)
  const transaction = await buildTransaction(
    /* request */ paymentRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(AxionUtils.getTransactionDigest(transaction)).toStrictEqual(testData.expectedPaymentDigest)

  const signedTransaction = await buildSignedTransaction(
    /* request */ paymentRequest,
    /* senderPrivKey */ testData.senderPrivateKey,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedPaymentSignature)
})

test('Futures deposit workflow', async () => {
  const admin = await getPublicKey(testData.adminPrivateKey)
  const sender = await getPublicKey(testData.senderPrivateKey)

  // TRANSACTION
  const depositRequest = buildAccountDepositRequest(1_000, admin)

  const transaction = await buildTransaction(
    /* request */ depositRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(AxionUtils.getTransactionDigest(transaction)).toStrictEqual(testData.expectedAccountDepositDigest)

  const signedTransaction = await buildSignedTransaction(
    /* request */ depositRequest,
    /* senderPrivKey */ testData.senderPrivateKey,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedAccountDepositSignature)
})

test('Futures withdrawal workflow', async () => {
  const admin = await getPublicKey(testData.adminPrivateKey)
  const sender = await getPublicKey(testData.senderPrivateKey)

  // TRANSACTIONS
  const withdrawalRequest = buildAccountWithdrawalRequest(1_000, admin)

  const transaction = await buildTransaction(
    /* request */ withdrawalRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(AxionUtils.getTransactionDigest(transaction)).toStrictEqual(testData.expectedAccountWithdrawalDigest)

  const signedTransaction = await buildSignedTransaction(
    /* request */ withdrawalRequest,
    /* senderPrivKey */ testData.senderPrivateKey,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedAccountWithdrawalSignature)
})

test('Futures limit order workflow', async () => {
  const admin = await getPublicKey(testData.adminPrivateKey)
  const sender = await getPublicKey(testData.senderPrivateKey)

  // TRANSACTIONS
  const orderRequest = buildFuturesLimitOrderRequest(0, 1, transactionParams.orderSide.Ask, 1_000, 1_000, admin)

  const transaction = await buildTransaction(
    /* request */ orderRequest,
    /* sender */ sender,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(AxionUtils.getTransactionDigest(transaction)).toStrictEqual(testData.expectedFuturesLimitOrderDigest)

  const signedTransaction = await buildSignedTransaction(
    /* request */ orderRequest,
    /* senderPrivKey */ testData.senderPrivateKey,
    /* defaultBlockDigest */ testData.defaultBlockDigest,
    /* fee */ undefined,
    /* client */ undefined
  )
  expect(signedTransaction.getSignature()).toStrictEqual(testData.expectedFuturesLimitOrderSignature)
})
