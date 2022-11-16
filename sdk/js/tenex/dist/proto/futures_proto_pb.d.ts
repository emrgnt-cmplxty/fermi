// package: futures_proto
// file: futures_proto.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";

export class CreateMarketplaceRequest extends jspb.Message { 
    getQuoteAssetId(): number;
    setQuoteAssetId(value: number): CreateMarketplaceRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CreateMarketplaceRequest.AsObject;
    static toObject(includeInstance: boolean, msg: CreateMarketplaceRequest): CreateMarketplaceRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CreateMarketplaceRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CreateMarketplaceRequest;
    static deserializeBinaryFromReader(message: CreateMarketplaceRequest, reader: jspb.BinaryReader): CreateMarketplaceRequest;
}

export namespace CreateMarketplaceRequest {
    export type AsObject = {
        quoteAssetId: number,
    }
}

export class CreateMarketRequest extends jspb.Message { 
    getBaseAssetId(): number;
    setBaseAssetId(value: number): CreateMarketRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CreateMarketRequest.AsObject;
    static toObject(includeInstance: boolean, msg: CreateMarketRequest): CreateMarketRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CreateMarketRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CreateMarketRequest;
    static deserializeBinaryFromReader(message: CreateMarketRequest, reader: jspb.BinaryReader): CreateMarketRequest;
}

export namespace CreateMarketRequest {
    export type AsObject = {
        baseAssetId: number,
    }
}

export class UpdateMarketParamsRequest extends jspb.Message { 
    getBaseAssetId(): number;
    setBaseAssetId(value: number): UpdateMarketParamsRequest;
    getMaxLeverage(): number;
    setMaxLeverage(value: number): UpdateMarketParamsRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): UpdateMarketParamsRequest.AsObject;
    static toObject(includeInstance: boolean, msg: UpdateMarketParamsRequest): UpdateMarketParamsRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: UpdateMarketParamsRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): UpdateMarketParamsRequest;
    static deserializeBinaryFromReader(message: UpdateMarketParamsRequest, reader: jspb.BinaryReader): UpdateMarketParamsRequest;
}

export namespace UpdateMarketParamsRequest {
    export type AsObject = {
        baseAssetId: number,
        maxLeverage: number,
    }
}

export class UpdateTimeRequest extends jspb.Message { 
    getLatestTime(): number;
    setLatestTime(value: number): UpdateTimeRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): UpdateTimeRequest.AsObject;
    static toObject(includeInstance: boolean, msg: UpdateTimeRequest): UpdateTimeRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: UpdateTimeRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): UpdateTimeRequest;
    static deserializeBinaryFromReader(message: UpdateTimeRequest, reader: jspb.BinaryReader): UpdateTimeRequest;
}

export namespace UpdateTimeRequest {
    export type AsObject = {
        latestTime: number,
    }
}

export class PriceEntry extends jspb.Message { 
    getAssetId(): number;
    setAssetId(value: number): PriceEntry;
    getPrice(): number;
    setPrice(value: number): PriceEntry;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PriceEntry.AsObject;
    static toObject(includeInstance: boolean, msg: PriceEntry): PriceEntry.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PriceEntry, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PriceEntry;
    static deserializeBinaryFromReader(message: PriceEntry, reader: jspb.BinaryReader): PriceEntry;
}

export namespace PriceEntry {
    export type AsObject = {
        assetId: number,
        price: number,
    }
}

export class UpdatePricesRequest extends jspb.Message { 
    clearPriceEntriesList(): void;
    getPriceEntriesList(): Array<PriceEntry>;
    setPriceEntriesList(value: Array<PriceEntry>): UpdatePricesRequest;
    addPriceEntries(value?: PriceEntry, index?: number): PriceEntry;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): UpdatePricesRequest.AsObject;
    static toObject(includeInstance: boolean, msg: UpdatePricesRequest): UpdatePricesRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: UpdatePricesRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): UpdatePricesRequest;
    static deserializeBinaryFromReader(message: UpdatePricesRequest, reader: jspb.BinaryReader): UpdatePricesRequest;
}

export namespace UpdatePricesRequest {
    export type AsObject = {
        priceEntriesList: Array<PriceEntry.AsObject>,
    }
}

export class AccountDepositRequest extends jspb.Message { 
    getQuantity(): number;
    setQuantity(value: number): AccountDepositRequest;
    getMarketAdmin(): Uint8Array | string;
    getMarketAdmin_asU8(): Uint8Array;
    getMarketAdmin_asB64(): string;
    setMarketAdmin(value: Uint8Array | string): AccountDepositRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccountDepositRequest.AsObject;
    static toObject(includeInstance: boolean, msg: AccountDepositRequest): AccountDepositRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccountDepositRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccountDepositRequest;
    static deserializeBinaryFromReader(message: AccountDepositRequest, reader: jspb.BinaryReader): AccountDepositRequest;
}

export namespace AccountDepositRequest {
    export type AsObject = {
        quantity: number,
        marketAdmin: Uint8Array | string,
    }
}

export class AccountWithdrawalRequest extends jspb.Message { 
    getQuantity(): number;
    setQuantity(value: number): AccountWithdrawalRequest;
    getMarketAdmin(): Uint8Array | string;
    getMarketAdmin_asU8(): Uint8Array;
    getMarketAdmin_asB64(): string;
    setMarketAdmin(value: Uint8Array | string): AccountWithdrawalRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccountWithdrawalRequest.AsObject;
    static toObject(includeInstance: boolean, msg: AccountWithdrawalRequest): AccountWithdrawalRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccountWithdrawalRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccountWithdrawalRequest;
    static deserializeBinaryFromReader(message: AccountWithdrawalRequest, reader: jspb.BinaryReader): AccountWithdrawalRequest;
}

export namespace AccountWithdrawalRequest {
    export type AsObject = {
        quantity: number,
        marketAdmin: Uint8Array | string,
    }
}

export class FuturesLimitOrderRequest extends jspb.Message { 
    getBaseAssetId(): number;
    setBaseAssetId(value: number): FuturesLimitOrderRequest;
    getQuoteAssetId(): number;
    setQuoteAssetId(value: number): FuturesLimitOrderRequest;
    getSide(): number;
    setSide(value: number): FuturesLimitOrderRequest;
    getPrice(): number;
    setPrice(value: number): FuturesLimitOrderRequest;
    getQuantity(): number;
    setQuantity(value: number): FuturesLimitOrderRequest;
    getMarketAdmin(): Uint8Array | string;
    getMarketAdmin_asU8(): Uint8Array;
    getMarketAdmin_asB64(): string;
    setMarketAdmin(value: Uint8Array | string): FuturesLimitOrderRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesLimitOrderRequest.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesLimitOrderRequest): FuturesLimitOrderRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesLimitOrderRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesLimitOrderRequest;
    static deserializeBinaryFromReader(message: FuturesLimitOrderRequest, reader: jspb.BinaryReader): FuturesLimitOrderRequest;
}

export namespace FuturesLimitOrderRequest {
    export type AsObject = {
        baseAssetId: number,
        quoteAssetId: number,
        side: number,
        price: number,
        quantity: number,
        marketAdmin: Uint8Array | string,
    }
}

export class CancelAllRequest extends jspb.Message { 
    getTarget(): Uint8Array | string;
    getTarget_asU8(): Uint8Array;
    getTarget_asB64(): string;
    setTarget(value: Uint8Array | string): CancelAllRequest;
    getMarketAdmin(): Uint8Array | string;
    getMarketAdmin_asU8(): Uint8Array;
    getMarketAdmin_asB64(): string;
    setMarketAdmin(value: Uint8Array | string): CancelAllRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CancelAllRequest.AsObject;
    static toObject(includeInstance: boolean, msg: CancelAllRequest): CancelAllRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CancelAllRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CancelAllRequest;
    static deserializeBinaryFromReader(message: CancelAllRequest, reader: jspb.BinaryReader): CancelAllRequest;
}

export namespace CancelAllRequest {
    export type AsObject = {
        target: Uint8Array | string,
        marketAdmin: Uint8Array | string,
    }
}

export class LiquidateRequest extends jspb.Message { 
    getBaseAssetId(): number;
    setBaseAssetId(value: number): LiquidateRequest;
    getQuoteAssetId(): number;
    setQuoteAssetId(value: number): LiquidateRequest;
    getSide(): number;
    setSide(value: number): LiquidateRequest;
    getQuantity(): number;
    setQuantity(value: number): LiquidateRequest;
    getMarketAdmin(): Uint8Array | string;
    getMarketAdmin_asU8(): Uint8Array;
    getMarketAdmin_asB64(): string;
    setMarketAdmin(value: Uint8Array | string): LiquidateRequest;
    getTarget(): Uint8Array | string;
    getTarget_asU8(): Uint8Array;
    getTarget_asB64(): string;
    setTarget(value: Uint8Array | string): LiquidateRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LiquidateRequest.AsObject;
    static toObject(includeInstance: boolean, msg: LiquidateRequest): LiquidateRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LiquidateRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LiquidateRequest;
    static deserializeBinaryFromReader(message: LiquidateRequest, reader: jspb.BinaryReader): LiquidateRequest;
}

export namespace LiquidateRequest {
    export type AsObject = {
        baseAssetId: number,
        quoteAssetId: number,
        side: number,
        quantity: number,
        marketAdmin: Uint8Array | string,
        target: Uint8Array | string,
    }
}

export class FuturesOrderNewEvent extends jspb.Message { 
    getAccount(): Uint8Array | string;
    getAccount_asU8(): Uint8Array;
    getAccount_asB64(): string;
    setAccount(value: Uint8Array | string): FuturesOrderNewEvent;
    getOrderId(): number;
    setOrderId(value: number): FuturesOrderNewEvent;
    getSide(): number;
    setSide(value: number): FuturesOrderNewEvent;
    getPrice(): number;
    setPrice(value: number): FuturesOrderNewEvent;
    getQuantity(): number;
    setQuantity(value: number): FuturesOrderNewEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesOrderNewEvent.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesOrderNewEvent): FuturesOrderNewEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesOrderNewEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesOrderNewEvent;
    static deserializeBinaryFromReader(message: FuturesOrderNewEvent, reader: jspb.BinaryReader): FuturesOrderNewEvent;
}

export namespace FuturesOrderNewEvent {
    export type AsObject = {
        account: Uint8Array | string,
        orderId: number,
        side: number,
        price: number,
        quantity: number,
    }
}

export class FuturesOrderFillEvent extends jspb.Message { 
    getAccount(): Uint8Array | string;
    getAccount_asU8(): Uint8Array;
    getAccount_asB64(): string;
    setAccount(value: Uint8Array | string): FuturesOrderFillEvent;
    getOrderId(): number;
    setOrderId(value: number): FuturesOrderFillEvent;
    getSide(): number;
    setSide(value: number): FuturesOrderFillEvent;
    getPrice(): number;
    setPrice(value: number): FuturesOrderFillEvent;
    getQuantity(): number;
    setQuantity(value: number): FuturesOrderFillEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesOrderFillEvent.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesOrderFillEvent): FuturesOrderFillEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesOrderFillEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesOrderFillEvent;
    static deserializeBinaryFromReader(message: FuturesOrderFillEvent, reader: jspb.BinaryReader): FuturesOrderFillEvent;
}

export namespace FuturesOrderFillEvent {
    export type AsObject = {
        account: Uint8Array | string,
        orderId: number,
        side: number,
        price: number,
        quantity: number,
    }
}

export class FuturesOrderPartialFillEvent extends jspb.Message { 
    getAccount(): Uint8Array | string;
    getAccount_asU8(): Uint8Array;
    getAccount_asB64(): string;
    setAccount(value: Uint8Array | string): FuturesOrderPartialFillEvent;
    getOrderId(): number;
    setOrderId(value: number): FuturesOrderPartialFillEvent;
    getSide(): number;
    setSide(value: number): FuturesOrderPartialFillEvent;
    getPrice(): number;
    setPrice(value: number): FuturesOrderPartialFillEvent;
    getQuantity(): number;
    setQuantity(value: number): FuturesOrderPartialFillEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesOrderPartialFillEvent.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesOrderPartialFillEvent): FuturesOrderPartialFillEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesOrderPartialFillEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesOrderPartialFillEvent;
    static deserializeBinaryFromReader(message: FuturesOrderPartialFillEvent, reader: jspb.BinaryReader): FuturesOrderPartialFillEvent;
}

export namespace FuturesOrderPartialFillEvent {
    export type AsObject = {
        account: Uint8Array | string,
        orderId: number,
        side: number,
        price: number,
        quantity: number,
    }
}

export class FuturesOrderUpdateEvent extends jspb.Message { 
    getAccount(): Uint8Array | string;
    getAccount_asU8(): Uint8Array;
    getAccount_asB64(): string;
    setAccount(value: Uint8Array | string): FuturesOrderUpdateEvent;
    getOrderId(): number;
    setOrderId(value: number): FuturesOrderUpdateEvent;
    getSide(): number;
    setSide(value: number): FuturesOrderUpdateEvent;
    getPrice(): number;
    setPrice(value: number): FuturesOrderUpdateEvent;
    getQuantity(): number;
    setQuantity(value: number): FuturesOrderUpdateEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesOrderUpdateEvent.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesOrderUpdateEvent): FuturesOrderUpdateEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesOrderUpdateEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesOrderUpdateEvent;
    static deserializeBinaryFromReader(message: FuturesOrderUpdateEvent, reader: jspb.BinaryReader): FuturesOrderUpdateEvent;
}

export namespace FuturesOrderUpdateEvent {
    export type AsObject = {
        account: Uint8Array | string,
        orderId: number,
        side: number,
        price: number,
        quantity: number,
    }
}

export class FuturesOrderCancelEvent extends jspb.Message { 
    getAccount(): Uint8Array | string;
    getAccount_asU8(): Uint8Array;
    getAccount_asB64(): string;
    setAccount(value: Uint8Array | string): FuturesOrderCancelEvent;
    getOrderId(): number;
    setOrderId(value: number): FuturesOrderCancelEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesOrderCancelEvent.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesOrderCancelEvent): FuturesOrderCancelEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesOrderCancelEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesOrderCancelEvent;
    static deserializeBinaryFromReader(message: FuturesOrderCancelEvent, reader: jspb.BinaryReader): FuturesOrderCancelEvent;
}

export namespace FuturesOrderCancelEvent {
    export type AsObject = {
        account: Uint8Array | string,
        orderId: number,
    }
}

export class FuturesLiquidateEvent extends jspb.Message { 
    getSender(): Uint8Array | string;
    getSender_asU8(): Uint8Array;
    getSender_asB64(): string;
    setSender(value: Uint8Array | string): FuturesLiquidateEvent;
    getTargetAccount(): Uint8Array | string;
    getTargetAccount_asU8(): Uint8Array;
    getTargetAccount_asB64(): string;
    setTargetAccount(value: Uint8Array | string): FuturesLiquidateEvent;
    getSide(): number;
    setSide(value: number): FuturesLiquidateEvent;
    getPrice(): number;
    setPrice(value: number): FuturesLiquidateEvent;
    getQuantity(): number;
    setQuantity(value: number): FuturesLiquidateEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FuturesLiquidateEvent.AsObject;
    static toObject(includeInstance: boolean, msg: FuturesLiquidateEvent): FuturesLiquidateEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FuturesLiquidateEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FuturesLiquidateEvent;
    static deserializeBinaryFromReader(message: FuturesLiquidateEvent, reader: jspb.BinaryReader): FuturesLiquidateEvent;
}

export namespace FuturesLiquidateEvent {
    export type AsObject = {
        sender: Uint8Array | string,
        targetAccount: Uint8Array | string,
        side: number,
        price: number,
        quantity: number,
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
    CANCEL_ORDER = 8,
    CANCEL_ALL = 9,
    LIQUIDATE = 10,
}

export enum FuturesEventType {
    ORDER_NEW = 0,
    ORDER_FILL = 1,
    ORDER_PARTIAL_FILL = 2,
    ORDER_UPDATE = 3,
    ORDER_CANCEL = 4,
    LIQUIDATE_EVENT = 5,
}
