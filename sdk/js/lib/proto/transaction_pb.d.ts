// package: transaction
// file: transaction.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from 'google-protobuf'

export class Version extends jspb.Message {
  getMajor(): number
  setMajor(value: number): Version
  getMinor(): number
  setMinor(value: number): Version
  getPatch(): number
  setPatch(value: number): Version

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): Version.AsObject
  static toObject(includeInstance: boolean, msg: Version): Version.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: Version, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): Version
  static deserializeBinaryFromReader(message: Version, reader: jspb.BinaryReader): Version
}

export namespace Version {
  export type AsObject = {
    major: number
    minor: number
    patch: number
  }
}

export class SignedTransaction extends jspb.Message {
  hasTransaction(): boolean
  clearTransaction(): void
  getTransaction(): Transaction | undefined
  setTransaction(value?: Transaction): SignedTransaction
  getSignature(): Uint8Array | string
  getSignature_asU8(): Uint8Array
  getSignature_asB64(): string
  setSignature(value: Uint8Array | string): SignedTransaction

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): SignedTransaction.AsObject
  static toObject(includeInstance: boolean, msg: SignedTransaction): SignedTransaction.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: SignedTransaction, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): SignedTransaction
  static deserializeBinaryFromReader(message: SignedTransaction, reader: jspb.BinaryReader): SignedTransaction
}

export namespace SignedTransaction {
  export type AsObject = {
    transaction?: Transaction.AsObject
    signature: Uint8Array | string
  }
}

export class Transaction extends jspb.Message {
  hasVersion(): boolean
  clearVersion(): void
  getVersion(): Version | undefined
  setVersion(value?: Version): Transaction
  getSender(): Uint8Array | string
  getSender_asU8(): Uint8Array
  getSender_asB64(): string
  setSender(value: Uint8Array | string): Transaction
  getTargetController(): number
  setTargetController(value: number): Transaction
  getRequestType(): number
  setRequestType(value: number): Transaction
  getRecentBlockHash(): Uint8Array | string
  getRecentBlockHash_asU8(): Uint8Array
  getRecentBlockHash_asB64(): string
  setRecentBlockHash(value: Uint8Array | string): Transaction
  getFee(): number
  setFee(value: number): Transaction
  getRequestBytes(): Uint8Array | string
  getRequestBytes_asU8(): Uint8Array
  getRequestBytes_asB64(): string
  setRequestBytes(value: Uint8Array | string): Transaction

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): Transaction.AsObject
  static toObject(includeInstance: boolean, msg: Transaction): Transaction.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: Transaction, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): Transaction
  static deserializeBinaryFromReader(message: Transaction, reader: jspb.BinaryReader): Transaction
}

export namespace Transaction {
  export type AsObject = {
    version?: Version.AsObject
    sender: Uint8Array | string
    targetController: number
    requestType: number
    recentBlockHash: Uint8Array | string
    fee: number
    requestBytes: Uint8Array | string
  }
}
