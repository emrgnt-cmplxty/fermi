// package: futures_requests
// file: futures_requests.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from 'google-protobuf'

export class CreateMarketplaceRequest extends jspb.Message {
  getQuoteAssetId(): number
  setQuoteAssetId(value: number): CreateMarketplaceRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): CreateMarketplaceRequest.AsObject
  static toObject(includeInstance: boolean, msg: CreateMarketplaceRequest): CreateMarketplaceRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: CreateMarketplaceRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): CreateMarketplaceRequest
  static deserializeBinaryFromReader(
    message: CreateMarketplaceRequest,
    reader: jspb.BinaryReader
  ): CreateMarketplaceRequest
}

export namespace CreateMarketplaceRequest {
  export type AsObject = {
    quoteAssetId: number
  }
}

export class CreateMarketRequest extends jspb.Message {
  getBaseAssetId(): number
  setBaseAssetId(value: number): CreateMarketRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): CreateMarketRequest.AsObject
  static toObject(includeInstance: boolean, msg: CreateMarketRequest): CreateMarketRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: CreateMarketRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): CreateMarketRequest
  static deserializeBinaryFromReader(message: CreateMarketRequest, reader: jspb.BinaryReader): CreateMarketRequest
}

export namespace CreateMarketRequest {
  export type AsObject = {
    baseAssetId: number
  }
}

export class UpdateMarketParamsRequest extends jspb.Message {
  getBaseAssetId(): number
  setBaseAssetId(value: number): UpdateMarketParamsRequest
  getMaxLeverage(): number
  setMaxLeverage(value: number): UpdateMarketParamsRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): UpdateMarketParamsRequest.AsObject
  static toObject(includeInstance: boolean, msg: UpdateMarketParamsRequest): UpdateMarketParamsRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: UpdateMarketParamsRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): UpdateMarketParamsRequest
  static deserializeBinaryFromReader(
    message: UpdateMarketParamsRequest,
    reader: jspb.BinaryReader
  ): UpdateMarketParamsRequest
}

export namespace UpdateMarketParamsRequest {
  export type AsObject = {
    baseAssetId: number
    maxLeverage: number
  }
}

export class UpdateTimeRequest extends jspb.Message {
  getLatestTime(): number
  setLatestTime(value: number): UpdateTimeRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): UpdateTimeRequest.AsObject
  static toObject(includeInstance: boolean, msg: UpdateTimeRequest): UpdateTimeRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: UpdateTimeRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): UpdateTimeRequest
  static deserializeBinaryFromReader(message: UpdateTimeRequest, reader: jspb.BinaryReader): UpdateTimeRequest
}

export namespace UpdateTimeRequest {
  export type AsObject = {
    latestTime: number
  }
}

export class UpdatePricesRequest extends jspb.Message {
  clearLatestPricesList(): void
  getLatestPricesList(): Array<number>
  setLatestPricesList(value: Array<number>): UpdatePricesRequest
  addLatestPrices(value: number, index?: number): number

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): UpdatePricesRequest.AsObject
  static toObject(includeInstance: boolean, msg: UpdatePricesRequest): UpdatePricesRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: UpdatePricesRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): UpdatePricesRequest
  static deserializeBinaryFromReader(message: UpdatePricesRequest, reader: jspb.BinaryReader): UpdatePricesRequest
}

export namespace UpdatePricesRequest {
  export type AsObject = {
    latestPricesList: Array<number>
  }
}

export class AccountDepositRequest extends jspb.Message {
  getQuantity(): number
  setQuantity(value: number): AccountDepositRequest
  getMarketAdmin(): Uint8Array | string
  getMarketAdmin_asU8(): Uint8Array
  getMarketAdmin_asB64(): string
  setMarketAdmin(value: Uint8Array | string): AccountDepositRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): AccountDepositRequest.AsObject
  static toObject(includeInstance: boolean, msg: AccountDepositRequest): AccountDepositRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: AccountDepositRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): AccountDepositRequest
  static deserializeBinaryFromReader(message: AccountDepositRequest, reader: jspb.BinaryReader): AccountDepositRequest
}

export namespace AccountDepositRequest {
  export type AsObject = {
    quantity: number
    marketAdmin: Uint8Array | string
  }
}

export class AccountWithdrawalRequest extends jspb.Message {
  getQuantity(): number
  setQuantity(value: number): AccountWithdrawalRequest
  getMarketAdmin(): Uint8Array | string
  getMarketAdmin_asU8(): Uint8Array
  getMarketAdmin_asB64(): string
  setMarketAdmin(value: Uint8Array | string): AccountWithdrawalRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): AccountWithdrawalRequest.AsObject
  static toObject(includeInstance: boolean, msg: AccountWithdrawalRequest): AccountWithdrawalRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: AccountWithdrawalRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): AccountWithdrawalRequest
  static deserializeBinaryFromReader(
    message: AccountWithdrawalRequest,
    reader: jspb.BinaryReader
  ): AccountWithdrawalRequest
}

export namespace AccountWithdrawalRequest {
  export type AsObject = {
    quantity: number
    marketAdmin: Uint8Array | string
  }
}

export class FuturesLimitOrderRequest extends jspb.Message {
  getBaseAssetId(): number
  setBaseAssetId(value: number): FuturesLimitOrderRequest
  getQuoteAssetId(): number
  setQuoteAssetId(value: number): FuturesLimitOrderRequest
  getSide(): number
  setSide(value: number): FuturesLimitOrderRequest
  getPrice(): number
  setPrice(value: number): FuturesLimitOrderRequest
  getQuantity(): number
  setQuantity(value: number): FuturesLimitOrderRequest
  getMarketAdmin(): Uint8Array | string
  getMarketAdmin_asU8(): Uint8Array
  getMarketAdmin_asB64(): string
  setMarketAdmin(value: Uint8Array | string): FuturesLimitOrderRequest

  serializeBinary(): Uint8Array
  toObject(includeInstance?: boolean): FuturesLimitOrderRequest.AsObject
  static toObject(includeInstance: boolean, msg: FuturesLimitOrderRequest): FuturesLimitOrderRequest.AsObject
  static extensions: { [key: number]: jspb.ExtensionFieldInfo<jspb.Message> }
  static extensionsBinary: { [key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message> }
  static serializeBinaryToWriter(message: FuturesLimitOrderRequest, writer: jspb.BinaryWriter): void
  static deserializeBinary(bytes: Uint8Array): FuturesLimitOrderRequest
  static deserializeBinaryFromReader(
    message: FuturesLimitOrderRequest,
    reader: jspb.BinaryReader
  ): FuturesLimitOrderRequest
}

export namespace FuturesLimitOrderRequest {
  export type AsObject = {
    baseAssetId: number
    quoteAssetId: number
    side: number
    price: number
    quantity: number
    marketAdmin: Uint8Array | string
  }
}

export enum FuturesRequestType {
  CREATE_MARKETPLACE = 0,
  CREATE_MARKET = 1,
  UPDATE_MARKET_PARAMS = 2,
  UPDATE_TIME = 3,
  UPDATE_PRICES = 4,
  ACCOUNT_DEPOSIT = 5,
  ACCOUNT_WITHDRAWAL = 6,
  FUTURES_LIMIT_ORDER = 7,
}
