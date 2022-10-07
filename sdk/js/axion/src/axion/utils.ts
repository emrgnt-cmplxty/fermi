// IMPORTS

// INTERNAL
import { types as AxionTypes } from './index'
import { blake2b } from 'blakejs'
import { Digest } from './types'
// EXPORTS

// GENERAL

export function delay(time) {
  return new Promise(resolve => setTimeout(resolve, time))
}

// Convert bytes to a hex string
export function bytesToHex(buffer: Digest, maxLength = 1e6): string {
  return (
    '0x' +
    [...new Uint8Array(buffer)]
      .map(x => x.toString(16).padStart(2, '0'))
      .join('')
      .slice(0, maxLength)
  )
}

// Convert a hex string to a byte array
export function hexToBytes(hex: string): Uint8Array {
  hex = hex.replace('0x', '')
  const bytes = []
  for (let c = 0; c < hex.length; c += 2) bytes.push(parseInt(hex.substr(c, 2), 16))
  return Uint8Array.from(bytes)
}

export function getTransactionDigest(transaction: AxionTypes.Transaction): Digest {
  return blake2b(transaction.serializeBinary(), undefined, 32)
}

export function getTransactionId(transaction: AxionTypes.Transaction): AxionTypes.TransactionId {
  return bytesToHex(getTransactionDigest(transaction), AxionTypes.transactionIdLen)
}

export function checkSubmissionResult(transaction: AxionTypes.QueriedTransaction) {
  const status = transaction.executed_transaction.result
  if (!Object.prototype.hasOwnProperty.call(status, 'Ok')) {
    throw Error(transaction.executed_transaction.result)
  }
}
