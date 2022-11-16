// IMPORTS

// INTERNAL
import { BankRequestType, CreateAssetRequest, PaymentRequest } from '../../dist/proto/bank_requests_pb'
import {
  FuturesRequestType,
  FuturesLimitOrderRequest,
  AccountDepositRequest,
  AccountWithdrawalRequest,
  UpdateMarketParamsRequest,
  CreateMarketRequest,
  CreateMarketplaceRequest,
  UpdateTimeRequest,
  PriceEntry,
  UpdatePricesRequest,
} from '../../dist/proto/futures_proto_pb'
import { Transaction, SignedTransaction, Version } from '../../dist/proto/transaction_pb'

// EXTERNAL
import assert from 'assert'

// EXPORTS
export {
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
  Transaction,
  SignedTransaction,
  Version,
}

// BANK CONTROLLER UTILITIES
export function buildPaymentRequest(receiver: Uint8Array, assetId: number, quantity: number): PaymentRequest {
  const paymentRequest = new PaymentRequest()
  paymentRequest.setReceiver(receiver)
  paymentRequest.setAssetId(assetId)
  paymentRequest.setQuantity(quantity)

  return paymentRequest
}

export function buildCreateAssetRequest(dummy: number): CreateAssetRequest {
  const createAssetRequest = new CreateAssetRequest()
  createAssetRequest.setDummy(dummy)

  return createAssetRequest
}

// FUTURES CONTROLLER UTILITIES
export function buildCreateMarketplaceRequest(quoteAssetId: number): CreateMarketplaceRequest {
  const createMarketplaceRequest = new CreateMarketplaceRequest()
  createMarketplaceRequest.setQuoteAssetId(quoteAssetId)

  return createMarketplaceRequest
}

export function buildCreateMarketRequest(baseAssetId: number): CreateMarketRequest {
  const createMarketRequest = new CreateMarketRequest()
  createMarketRequest.setBaseAssetId(baseAssetId)

  return createMarketRequest
}

export function buildUpdateMarketParamsRequest(baseAssetId: number, maxLeverage: number): UpdateMarketParamsRequest {
  const updateMarketParamsRequest = new UpdateMarketParamsRequest()
  updateMarketParamsRequest.setBaseAssetId(baseAssetId)
  updateMarketParamsRequest.setMaxLeverage(maxLeverage)

  return updateMarketParamsRequest
}

export function buildUpdateTimeRequest(latest_time: number): UpdateTimeRequest {
  const updateTimeRequest = new UpdateTimeRequest()
  updateTimeRequest.setLatestTime(latest_time)

  return updateTimeRequest
}

export function buildUpdatePricesRequest(asset_ids_prices: number[][]): UpdatePricesRequest {
  const updatePricesRequest = new UpdatePricesRequest()
  const priceEntries = []
  for (const asset_id_price of asset_ids_prices) {
    const priceEntry = new PriceEntry()
    priceEntry.setAssetId(asset_id_price[0])
    priceEntry.setPrice(asset_id_price[1])
    priceEntries.push(priceEntry)
  }
  updatePricesRequest.setPriceEntriesList(priceEntries)

  return updatePricesRequest
}

export function buildAccountDepositRequest(quantity: number, marketAdmin: Uint8Array): AccountDepositRequest {
  const accountDepositRequest = new AccountDepositRequest()
  accountDepositRequest.setQuantity(quantity)
  accountDepositRequest.setMarketAdmin(marketAdmin)

  return accountDepositRequest
}

export function buildAccountWithdrawalRequest(quantity: number, marketAdmin: Uint8Array): AccountWithdrawalRequest {
  const accountWithdrawalRequest = new AccountWithdrawalRequest()
  accountWithdrawalRequest.setQuantity(quantity)
  accountWithdrawalRequest.setMarketAdmin(marketAdmin)

  return accountWithdrawalRequest
}

export function buildFuturesLimitOrderRequest(
  baseAssetId: number,
  quoteAssetId: number,
  side: number,
  price: number,
  quantity: number,
  admin: Uint8Array
): FuturesLimitOrderRequest {
  const futuresLimitOrderRequest = new FuturesLimitOrderRequest()
  futuresLimitOrderRequest.setBaseAssetId(baseAssetId)
  futuresLimitOrderRequest.setQuoteAssetId(quoteAssetId)
  assert(side == 1 || side == 2)
  futuresLimitOrderRequest.setSide(side)
  futuresLimitOrderRequest.setPrice(price)
  futuresLimitOrderRequest.setQuantity(quantity)
  futuresLimitOrderRequest.setMarketAdmin(admin)

  return futuresLimitOrderRequest
}
