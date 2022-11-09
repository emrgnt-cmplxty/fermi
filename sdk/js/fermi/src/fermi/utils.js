'use strict'
// IMPORTS
var __spreadArray =
  (this && this.__spreadArray) ||
  function (to, from, pack) {
    if (pack || arguments.length === 2)
      for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
          if (!ar) ar = Array.prototype.slice.call(from, 0, i)
          ar[i] = from[i]
        }
      }
    return to.concat(ar || Array.prototype.slice.call(from))
  }
exports.__esModule = true
exports.checkSubmissionResult =
  exports.getTransactionId =
  exports.getTransactionDigest =
  exports.hexToBytes =
  exports.bytesToHex =
  exports.delay =
    void 0
// INTERNAL
var index_1 = require('./index')
var blakejs_1 = require('blakejs')
// EXPORTS
// GENERAL
function delay(time) {
  return new Promise(function (resolve) {
    return setTimeout(resolve, time)
  })
}
exports.delay = delay
// Convert bytes to a hex string
function bytesToHex(buffer, maxLength) {
  if (maxLength === void 0) {
    maxLength = 1e6
  }
  return (
    '0x' +
    __spreadArray([], new Uint8Array(buffer), true)
      .map(function (x) {
        return x.toString(16).padStart(2, '0')
      })
      .join('')
      .slice(0, maxLength)
  )
}
exports.bytesToHex = bytesToHex
// Convert a hex string to a byte array
function hexToBytes(hex) {
  hex = hex.replace('0x', '')
  var bytes = []
  for (var c = 0; c < hex.length; c += 2) bytes.push(parseInt(hex.substr(c, 2), 16))
  return Uint8Array.from(bytes)
}
exports.hexToBytes = hexToBytes
function getTransactionDigest(transaction) {
  return (0, blakejs_1.blake2b)(transaction.serializeBinary(), undefined, 32)
}
exports.getTransactionDigest = getTransactionDigest
function getTransactionId(transaction) {
  return bytesToHex(getTransactionDigest(transaction), index_1.types.transactionIdLen)
}
exports.getTransactionId = getTransactionId
function checkSubmissionResult(transaction) {
  var status = transaction.executed_transaction.result
  if (!Object.prototype.hasOwnProperty.call(status, 'Ok')) {
    throw Error(transaction.executed_transaction.result)
  }
}
exports.checkSubmissionResult = checkSubmissionResult
