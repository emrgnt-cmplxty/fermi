import * as grpc from '@grpc/grpc-js'
import {
  RelayerGetBlockInfoRequest,
  RelayerGetBlockRequest,
  RelayerGetLatestBlockInfoRequest,
  RelayerBlockInfoResponse,
  RelayerBlockResponse,
  Empty,
} from '../../dist/proto/services_pb'
import { RelayerClient, TransactionSubmitterClient } from '../../dist/proto/services_grpc_pb'
import { Transaction, SignedTransaction, Version } from '../../dist/proto/transaction_pb'
import {
  digest,
  privateKey,
  transactionId,
  executedTransaction,
  getTransactionDigest,
  getTransactionId,
  lookupControllerAndType,
  transactionParams,
  supportedOrders,
  waitForConfirmation,
} from '../utils'
import { sign, getPublicKey } from '@noble/ed25519'

export default class TenexClient {
  public relayerClient: RelayerClient
  public transactionClient: TransactionSubmitterClient

  constructor(relayerAddress: string, validatorAddress: string) {
    this.relayerClient = new RelayerClient(relayerAddress, grpc.credentials.createInsecure())
    this.transactionClient = new TransactionSubmitterClient(validatorAddress, grpc.credentials.createInsecure())
  }

  submitTransaction(transaction: SignedTransaction): Promise<Empty> {
    return new Promise<Empty>((resolve, reject) => {
      this.transactionClient.submitTransaction(transaction, (err, empty) => {
        if (err) {
          return reject(err)
        }
        return resolve(empty)
      })
    })
  }

  getBlockInfo(blockNumber: number): Promise<RelayerBlockInfoResponse> {
    return new Promise<RelayerBlockInfoResponse>((resolve, reject) => {
      this.relayerClient.getBlockInfo(
        new RelayerGetBlockInfoRequest().setBlockNumber(blockNumber),
        (err, blockInfo) => {
          if (err) {
            return reject(err)
          }
          return resolve(blockInfo)
        }
      )
    })
  }

  getLatestBlockInfo(): Promise<RelayerBlockInfoResponse> {
    return new Promise<RelayerBlockInfoResponse>((resolve, reject) => {
      this.relayerClient.getLatestBlockInfo(new RelayerGetLatestBlockInfoRequest(), (err, blockInfo) => {
        if (err) {
          return reject(err)
        }
        return resolve(blockInfo)
      })
    })
  }

  getBlock(blockNumber: number): Promise<RelayerBlockResponse> {
    return new Promise<RelayerBlockResponse>((resolve, reject) => {
      const request = new RelayerGetBlockRequest().setBlockNumber(blockNumber)
      this.relayerClient.getBlock(request, (err, block) => {
        if (err) {
          return reject(err)
        }
        return resolve(block)
      })
    })
  }

  async buildTransaction(
    request: supportedOrders,
    sender: privateKey,
    recentBlockDigest: digest | undefined,
    fee: number | undefined
  ): Promise<Transaction> {
    const transaction = new Transaction()
    const version = new Version()
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
      const blockInfoResponse = await this.getLatestBlockInfo()
      const blockInfo = blockInfoResponse.getBlockInfo()
      transaction.setRecentBlockHash(blockInfo.getDigest())
    }
    transaction.setRequestType(requestType)
    transaction.setFee(fee == undefined ? transactionParams.minFee : fee)
    transaction.setRequestBytes(request.serializeBinary())

    return transaction
  }

  async buildSignedTransaction(
    request: supportedOrders,
    senderPrivKey: privateKey,
    recentBlockDigest: digest | undefined,
    fee: number | undefined
  ): Promise<SignedTransaction> {
    const sender = await getPublicKey(senderPrivKey)
    const transaction = await this.buildTransaction(request, sender, recentBlockDigest, fee)
    const transactionDigest = getTransactionDigest(transaction)

    const signedTransaction = new SignedTransaction()
    signedTransaction.setTransaction(transaction)
    const signature = await sign(transactionDigest, senderPrivKey)
    signedTransaction.setSignature(signature)

    return signedTransaction
  }

  async sendTransaction(signedTransaction: SignedTransaction): Promise<transactionId> {
    await this.submitTransaction(signedTransaction)
    const calcTransactionId = getTransactionId(signedTransaction.getTransaction())
    return calcTransactionId
  }

  async sendAndConfirmTransaction(signedTransaction: SignedTransaction): Promise<executedTransaction> {
    const calcTransactionId = getTransactionId(signedTransaction.getTransaction())
    await this.sendTransaction(signedTransaction)
    const submitTransactionResult = await waitForConfirmation(this, calcTransactionId)
    return new executedTransaction(calcTransactionId, submitTransactionResult)
  }
}
