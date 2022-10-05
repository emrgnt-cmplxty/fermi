// IMPORTS

// INTERNAL
import { delay, bytesToHex, getTransactionId } from './utils'
import {
  Block,
  BlockInfo,
  ExecutedTransaction,
  QueriedTransaction,
  Response,
  SignedTransaction,
  TransactionId,
  TransactionResponse,
  successResponse,
} from './types'

// EXTERNAL
import fetch, { Response as FetchResponse } from 'node-fetch'

// The AxionClient can be used to interact with the basic functionality of any Axion blockchain.
// These functions include the methods getLatestBlockInfo, getBlockInfo, getBlock, and submitTransaction.
// It is assumed that any Axion blockchain will have these methods.
export default class AxionClient {
  public jsonrpcURI: string
  public namespace: string

  constructor(jsonrpcUri: string, namespace = 'tenex') {
    this.jsonrpcURI = jsonrpcUri
    this.namespace = namespace
  }

  getRequestBody(method: string, params: string): string {
    return `{"jsonrpc":"2.0", "id":"1", "method":"${method}", "params":${params}}`
  }

  async request<T>(method: string, params = '[]'): Promise<Response<T>> {
    const body = this.getRequestBody(method, params)
    const response: FetchResponse = await fetch(this.jsonrpcURI, {
      body: body,
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
      method: 'POST',
    })

    const jsonResponse = (await response.json()) as Response<T>
    return jsonResponse
  }

  async getLatestBlockInfo(): Promise<BlockInfo> {
    const response: Response<BlockInfo> = await this.request(`${this.namespace}_getLatestBlockInfo`)
    return response.result
  }

  async getBlockInfo(blockNumber: number): Promise<BlockInfo> {
    const response: Response<BlockInfo> = await this.request(`${this.namespace}_getBlockInfo`, `[${blockNumber}]`)
    return response.result
  }

  async getBlock(blockNumber: number): Promise<Block> {
    const response: Response<Block> = await this.request(`${this.namespace}_getBlock`, `[${blockNumber}]`)
    return response.result
  }

  async submitTransaction(signedTransaction: SignedTransaction): Promise<TransactionId> {
    if (signedTransaction.getTransaction() == undefined) {
      throw new Error('Transaction is undefined')
    }

    const transactionHex = bytesToHex(signedTransaction.serializeBinary())
    const submitResult: Response<TransactionResponse> = await this.request(
      `${this.namespace}_submitTransaction`,
      `["${transactionHex}"]`
    )

    if (submitResult.result !== successResponse) {
      throw new Error(submitResult.result)
    }
    const transaction = signedTransaction.getTransaction()
    if (transaction == undefined) {
      throw new Error('Transaction is undefined')
    }
    const calcTransactionId = getTransactionId(transaction)
    return calcTransactionId
  }

  private async processNewBlock(digestContainer: Record<string, ExecutedTransaction>, latestBlockNumber: number) {
    const block: Block = await this.getBlock(latestBlockNumber)
    for (let i = 0; i < block.transactions.length; i++) {
      const executedTransaction = block.transactions[i]
      digestContainer[executedTransaction.transaction_id] = executedTransaction.executed_transaction
    }
  }

  private async waitForConfirmation(
    transactionId: TransactionId,
    maxNumberOfBlocks = 100, // TODO - reduce wait time for testnet/mainnet, keep high for local devnet
    delayTime = 500
  ): Promise<QueriedTransaction> {
    let blockCounter = 0
    const digestContainer: Record<string, ExecutedTransaction> = {}

    let latestBlockInfo = await this.getLatestBlockInfo()
    let latestBlockNumber = latestBlockInfo.block_number

    await this.processNewBlock(digestContainer, latestBlockNumber)

    while (blockCounter < maxNumberOfBlocks) {
      const latestBlockInfo = await this.getLatestBlockInfo()

      if (latestBlockNumber != latestBlockInfo.block_number) {
        // Fetch all blocks between last observed block and current block
        // This is mostly necessary for the case where the node is behind
        // Or when blocks are being produced very quickly in local devnet
        for (let i = latestBlockNumber + 1; i <= latestBlockInfo.block_number; i++) {
          await this.processNewBlock(digestContainer, i)
          blockCounter++
        }
        latestBlockNumber = latestBlockInfo.block_number
      }

      if (digestContainer[transactionId]) {
        return {
          executed_transaction: digestContainer[transactionId],
          transaction_id: transactionId,
        } as QueriedTransaction
      }
      await delay(delayTime)
    }
    latestBlockInfo = await this.getLatestBlockInfo()
    latestBlockNumber = latestBlockInfo.block_number
    throw new Error('Failed to confirm transaction')
  }

  async sendAndConfirmTransaction(signedTransaction: SignedTransaction): Promise<QueriedTransaction> {
    const transaction = signedTransaction.getTransaction()
    if (transaction == undefined) {
      throw new Error('Transaction is undefined')
    }
    const calcTransactionId = getTransactionId(transaction)
    await this.submitTransaction(signedTransaction)
    return await this.waitForConfirmation(calcTransactionId)
  }
}
