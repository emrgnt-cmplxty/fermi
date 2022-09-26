import assert from 'assert'
import {
  CreateMarketplaceRequest,
  CreateMarketRequest,
  UpdateMarketParamsRequest,
  UpdatePricesRequest,
  UpdateTimeRequest,
  AccountDepositRequest,
  AccountWithdrawalRequest,
  FuturesLimitOrderRequest,
} from '../../dist/proto/futures_requests_pb'

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

export function buildUpdatePricesRequest(latest_prices: number[]): UpdatePricesRequest {
  const updatePricesRequest = new UpdatePricesRequest()
  updatePricesRequest.setLatestPricesList(latest_prices)

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
