// IMPORTS

// INTERNAL
import { FermiClient, FermiTypes } from 'fermi-js-sdk'
import { MarketplaceResponse, MarketResponse, MarketplaceUserInfoResponse, OrderbookDepthResponse } from './types'

export default class TenexClient extends FermiClient {
  public jsonrpcURI: string
  public namespace: string

  constructor(jsonrpcUri: string, namespace = 'tenex') {
    super(jsonrpcUri)
    this.jsonrpcURI = jsonrpcUri
    this.namespace = namespace
  }

  async getFuturesMarketPlaces(): Promise<MarketplaceResponse[]> {
    const response: FermiTypes.Response<MarketplaceResponse[]> = await this.request(
      `${this.namespace}_getFuturesMarketplaces`
    )
    return response.result
  }

  async getFuturesMarkets(marketAdmin: string): Promise<MarketResponse[]> {
    const response: FermiTypes.Response<MarketResponse[]> = await this.request(
      `${this.namespace}_getMarkets`,
      `["${marketAdmin}"]`
    )
    return response.result
  }

  async getUserMarketplaceInfo(marketAdmin: string, user: string): Promise<MarketplaceUserInfoResponse> {
    const response: FermiTypes.Response<MarketplaceUserInfoResponse> = await this.request(
      `${this.namespace}_getUserMarketplaceInfo`,
      `["${marketAdmin}", "${user}"]`
    )
    return response.result
  }


  async getOrderbookDepth(marketAdmin: string, baseAssetId: number, quoteAssetId: number, depth: number): Promise<OrderbookDepthResponse> {
    if (depth > 100) {
      throw new Error("Maximum order book depth exceeded")
    }
    const response: FermiTypes.Response<OrderbookDepthResponse> = await this.request(
      `${this.namespace}_getOrderbookDepth`,
      `["${marketAdmin}", ${baseAssetId}, ${quoteAssetId}, ${depth}]`
    )
    return response.result
  }

}