// IMPORTS

// INTERNAL
import {
  BankRequestType,
  CreateAssetRequest,
  PaymentRequest,
  FuturesRequestType,
  FuturesLimitOrderRequest,
  AccountDepositRequest,
  AccountWithdrawalRequest,
  UpdateMarketParamsRequest,
  CreateMarketRequest,
  CreateMarketplaceRequest,
  UpdateTimeRequest,
  UpdatePricesRequest,
} from './transaction'
import { types as AxionTypes, utils as AxionUtils, account as AxionAccount, AxionClient } from '../axion'

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

type bankRequests = PaymentRequest | CreateAssetRequest
type futuresRequests =
  | CreateMarketplaceRequest
  | CreateMarketRequest
  | UpdateMarketParamsRequest
  | UpdateTimeRequest
  | UpdatePricesRequest
  | AccountDepositRequest
  | FuturesLimitOrderRequest
  | AccountWithdrawalRequest

export type supportedOrders = bankRequests | futuresRequests

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

export async function buildTransaction(
  request: supportedOrders,
  sender: AxionTypes.PrivateKey,
  recentBlockDigest: AxionTypes.Digest | undefined,
  fee: number | undefined,
  client: AxionClient | undefined
): Promise<AxionTypes.Transaction> {
  const transaction = new AxionTypes.Transaction()
  const version = new AxionTypes.Version()
  const [targetController, requestType] = lookupControllerAndType(request)

  version.setMajor(transactionParams.protoVersion.major)
  version.setMinor(transactionParams.protoVersion.minor)
  version.setPatch(transactionParams.protoVersion.patch)
  transaction.setVersion(version)
  transaction.setSender(sender)
  transaction.setTargetController(targetController)
  if (recentBlockDigest !== undefined) {
    transaction.setRecentBlockHash(recentBlockDigest)
  } else {
    if (client === undefined) {
      throw Error('Client must be defined if recentBlockDigest is not')
    }
    const blockInfoResponse: AxionTypes.BlockInfo = await client.getLatestBlockInfo()
    const recentBlockDigest = blockInfoResponse.block_id
    transaction.setRecentBlockHash(AxionUtils.hexToBytes(recentBlockDigest))
  }
  transaction.setRequestType(requestType)
  transaction.setFee(fee == undefined ? transactionParams.minFee : fee)
  transaction.setRequestBytes(request.serializeBinary())

  return transaction
}

export async function buildSignedTransaction(
  request: supportedOrders,
  senderPrivKey: AxionTypes.PrivateKey,
  recentBlockDigest: AxionTypes.Digest | undefined,
  fee: number | undefined,
  client: AxionClient | undefined
): Promise<AxionTypes.SignedTransaction> {
  const sender = await AxionAccount.getPublicKey(senderPrivKey)
  const transaction = await buildTransaction(request, sender, recentBlockDigest, fee, client)
  const transactionDigest = AxionUtils.getTransactionDigest(transaction)

  const signedTransaction = new AxionTypes.SignedTransaction()
  signedTransaction.setTransaction(transaction)
  const signature = await AxionAccount.sign(transactionDigest, senderPrivKey)
  signedTransaction.setSignature(signature)

  return signedTransaction
}
