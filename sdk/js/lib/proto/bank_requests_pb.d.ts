// package: bank_requests
// file: bank_requests.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from 'google-protobuf'

export class CreateAssetRequest extends jspb.Message {
  getDummy(): number
  setDummy(value: number): CreateAssetRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): CreateAssetRequest.AsObject
  static toObject(includeInstance: boolean, msg: CreateAssetRequest): CreateAssetRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: CreateAssetRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): CreateAssetRequest
  static deserializeBinaryFromReader(message: CreateAssetRequest, reader: jspb.BinaryReader): CreateAssetRequest
}

export namespace CreateAssetRequest {
  export type AsObject = {
    dummy: number
  }
}

export class PaymentRequest extends jspb.Message {
  getReceiver(): Uint8Array | string
  getReceiver_asU8(): Uint8Array
  getReceiver_asB64(): string
  setReceiver(value: Uint8Array | string): PaymentRequest
  getAssetId(): number
  setAssetId(value: number): PaymentRequest
  getQuantity(): number
  setQuantity(value: number): PaymentRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): PaymentRequest.AsObject
  static toObject(includeInstance: boolean, msg: PaymentRequest): PaymentRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: PaymentRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): PaymentRequest
  static deserializeBinaryFromReader(message: PaymentRequest, reader: jspb.BinaryReader): PaymentRequest
}

export namespace PaymentRequest {
  export type AsObject = {
    receiver: Uint8Array | string
    assetId: number
    quantity: number
  }
}

export enum BankRequestType {
  CREATE_ASSET = 0,
  PAYMENT = 1,
}
