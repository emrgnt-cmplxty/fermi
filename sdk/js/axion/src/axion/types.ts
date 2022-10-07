// IMPORTS

// INTERNAL
import { Transaction, SignedTransaction, Version } from '../../dist/proto/transaction_pb'

// TYPES

type Response<T> = {
  jsonrpc: string
  result: T
  id: string
}

// camel casing is used to match the JSON RPC spec
type BlockInfo = {
  validator_system_epoch_time_in_micros: number
  block_number: number
  block_id: string
}

type ExecutedTransaction = {
  signed_transaction: Uint8Array
  events: string[]
  result: string
}

type QueriedTransaction = {
  executed_transaction: ExecutedTransaction
  transaction_id: string
}

type Block = {
  transactions: QueriedTransaction[]
  block_id: string
}

type Digest = Uint8Array

type PrivateKey = Uint8Array

type PublicKey = Uint8Array

type TransactionResponse = string

type TransactionId = string

const successResponse = 'Success'

const transactionIdLen = 64

// EXPORTS

export {
  Response,
  BlockInfo,
  ExecutedTransaction,
  QueriedTransaction,
  Block,
  Digest,
  PrivateKey,
  PublicKey,
  TransactionResponse,
  TransactionId,
  Transaction,
  SignedTransaction,
  Version,
  successResponse,
  transactionIdLen,
}
