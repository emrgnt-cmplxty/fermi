// IMPORTS
import {
  FuturesRequestType,
  FuturesLimitOrderRequest,
  AccountDepositRequest,
  AccountWithdrawalRequest,
  UpdateMarketParamsRequest,
  CreateMarketRequest,
  CreateMarketplaceRequest,
  UpdateTimeRequest,
  UpdatePricesRequest,
} from '../../dist/proto/futures_requests_pb'
import { BankRequestType, CreateAssetRequest, PaymentRequest } from '../../dist/proto/bank_requests_pb'
import { SignedTransaction, Transaction } from '../../dist/proto/transaction_pb'
export * from './bankController'
export * from './futuresController'
import { TenexClient } from '../'
import { blake2b } from 'blakejs'

export type transactionId = string
export type transactionResult = string
export class executedTransaction {
  public transactionId: transactionId
  public transactionResult: transactionResult
  constructor(transactionId: transactionId, transactionResult: transactionResult) {
    this.transactionId = transactionId
    this.transactionResult = transactionResult
  }
}
export type digest = Uint8Array
export type privateKey = Uint8Array

const ADDR_LEN = 20
const TRANSACTION_ID_LEN = 64

export const transactionParams = {
  protoVersion: {
    major: 0,
    minor: 0,
    patch: 0,
  },
  minFee: 1_000,
  controllerType: {
    Bank: 0,
    Stake: 1,
    Spot: 2,
    Consensus: 3,
    Futures: 4,
  },
  orderSide: {
    Bid: 1,
    Ask: 2,
  },
}

type futuresRequests =
  | CreateMarketplaceRequest
  | CreateMarketRequest
  | UpdateMarketParamsRequest
  | UpdateTimeRequest
  | UpdatePricesRequest
  | AccountDepositRequest
  | FuturesLimitOrderRequest
  | AccountWithdrawalRequest
export type supportedOrders = PaymentRequest | CreateAssetRequest | futuresRequests

export function lookupControllerAndType(request: supportedOrders): [number, number] {
  if (request instanceof PaymentRequest) {
    return [transactionParams.controllerType.Bank, BankRequestType.PAYMENT]
  } else if (request instanceof CreateAssetRequest) {
    return [transactionParams.controllerType.Bank, BankRequestType.CREATE_ASSET]
  } else if (request instanceof CreateMarketplaceRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.CREATE_MARKETPLACE]
  } else if (request instanceof CreateMarketRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.CREATE_MARKET]
  } else if (request instanceof UpdateMarketParamsRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.UPDATE_MARKET_PARAMS]
  } else if (request instanceof UpdateTimeRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.UPDATE_TIME]
  } else if (request instanceof UpdatePricesRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.UPDATE_PRICES]
  } else if (request instanceof AccountDepositRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.ACCOUNT_DEPOSIT]
  } else if (request instanceof FuturesLimitOrderRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.FUTURES_LIMIT_ORDER]
  } else if (request instanceof AccountWithdrawalRequest) {
    return [transactionParams.controllerType.Futures, FuturesRequestType.ACCOUNT_WITHDRAWAL]
  } else {
    throw new Error('Unsupported request type')
  }
}

export function bytesToHex(buffer: digest, maxLength = ADDR_LEN): string {
  return (
    '0x' +
    [...new Uint8Array(buffer)]
      .map(x => x.toString(16).padStart(2, '0'))
      .join('')
      .slice(0, maxLength)
  )
}

export function getTransactionDigest(transaction: Transaction): digest {
  return blake2b(transaction.serializeBinary(), undefined, 32)
}

export function getTransactionId(transaction: Transaction): transactionId {
  return bytesToHex(getTransactionDigest(transaction), TRANSACTION_ID_LEN)
}

function delay(time) {
  return new Promise(resolve => setTimeout(resolve, time))
}

function insertDigests(digestContainer: any, execResults: [digest, transactionResult][]) {
  for (let i = 0; i < execResults.length; i++) {
    let transactionDigest = execResults[i][0]
    let transactionResult = execResults[i][1]
    digestContainer[bytesToHex(transactionDigest, TRANSACTION_ID_LEN)] = transactionResult
  }
}

async function processNewBlock(client: TenexClient, digestContainer: any, latestBlockNumber: number) {
  let blockResponse = await client.getBlock(latestBlockNumber)
  let block = await blockResponse.getBlock()
  let execResults: [digest, string][] = []
  for (let i = 0; i < block.getExecutedTransactionsList().length; i++) {
    let executed_transaction = block.getExecutedTransactionsList()[i]
    let digest = executed_transaction.getDigest() as digest
    execResults.push([digest, executed_transaction.getResult()])
  }

  insertDigests(digestContainer, execResults)
}

export async function waitForConfirmation(
  client: TenexClient,
  transactionId: transactionId,
  maxNumberOfBlocks = 10,
  delayTime = 500
): Promise<transactionResult> {
  let blockCounter = 0
  let digestContainer = {}

  let blockInfoResponse = await client.getLatestBlockInfo()
  let blockInfo = blockInfoResponse.getBlockInfo()
  let latestBlockNumber = blockInfo.getBlockNumber()
  await processNewBlock(client, digestContainer, latestBlockNumber)

  while (blockCounter < maxNumberOfBlocks) {
    let blockInfoResponse = await client.getLatestBlockInfo()
    let blockInfo = blockInfoResponse.getBlockInfo()

    if (latestBlockNumber != blockInfo.getBlockNumber()) {
      // fetch all blocks between last observed block and current block
      // this is mostly necessary for the case where the node is behind
      // or when blocks are being produced very quickly in local devnet
      for (let i = latestBlockNumber + 1; i <= blockInfo.getBlockNumber(); i++) {
        await processNewBlock(client, digestContainer, i)
      }
      latestBlockNumber = blockInfo.getBlockNumber()
      blockCounter++
    }

    if (digestContainer[transactionId]) {
      return digestContainer[transactionId]
    }
    await delay(delayTime)
  }
  throw new Error('Failed to confirm transaction')
}
