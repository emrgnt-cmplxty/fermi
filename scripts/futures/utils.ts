import { FermiClient, FermiTypes, FermiUtils, FermiAccount } from 'fermi-js-sdk'
import { TenexTransaction, TenexUtils } from 'tenex-js-sdk'

export function getJsonRpcUrl(authority): string {
    // construct the URL dynamically from the multiaddr
    let link = authority['jsonrpc_address'].split('/')[2]
    let port = authority['jsonrpc_address'].split('/')[4]
    let url = "http://" + link + ":" + port
    return url
}

export class SDKAdapter {
    public privateKey: Uint8Array
    public client: FermiClient
  
    constructor(privateKey: Uint8Array, client: FermiClient) {
      this.privateKey = privateKey
      this.client = client
    }

    async sendCreateAsset(dummy: number) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildCreateAssetRequest(dummy),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending asset creation tranasction')
        const result: FermiTypes.QueriedTransaction = await this.client.sendAndConfirmTransaction(signedTransaction)
        console.log('result=', result)
        FermiUtils.checkSubmissionResult(result)
    }

    async sendCreateMarketplaceRequest(quoteAssetId: number) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildCreateMarketplaceRequest(quoteAssetId),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending create marketplace tranasction')
        const result: FermiTypes.QueriedTransaction = await this.client.sendAndConfirmTransaction(signedTransaction)
        console.log('result=', result)
        FermiUtils.checkSubmissionResult(result)
    }

    async sendCreateMarketRequest(baseAssetId: number) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildCreateMarketRequest(baseAssetId),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending create market tranasction')
        const result: FermiTypes.QueriedTransaction = await this.client.sendAndConfirmTransaction(signedTransaction)
        console.log('result=', result)
        FermiUtils.checkSubmissionResult(result)
    }

    async sendAccountDepositRequest(quantity: number, marketAdmin: Uint8Array) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildAccountDepositRequest(quantity, marketAdmin),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending create account deposit tranasction')
        const result: FermiTypes.QueriedTransaction = await this.client.sendAndConfirmTransaction(signedTransaction)
        console.log('result=', result)
        FermiUtils.checkSubmissionResult(result)
    }

    async sendFuturesLimitOrderRequest(baseAssetId: number, quoteAssetId: number, side: number, price: number, quantity: number, marketAdmin: Uint8Array) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildFuturesLimitOrderRequest(baseAssetId, quoteAssetId, side, price, quantity, marketAdmin),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending futures limit order tranasction')
        await this.client.submitTransaction(signedTransaction)
        // const result: FermiTypes.QueriedTransaction = await this.client.sendAndConfirmTransaction(signedTransaction)
        // console.log('result=', result)
        // console.log('result.executed_transaction.events=', result.executed_transaction.events)
        // FermiUtils.checkSubmissionResult(result)
    }

    async sendUpdatePricesRequest(assetIdsPrices: number[][]) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildUpdatePricesRequest(assetIdsPrices),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending update prices tranasction')
        const result: FermiTypes.QueriedTransaction = await this.client.sendAndConfirmTransaction(signedTransaction)
        console.log('result=', result)
        FermiUtils.checkSubmissionResult(result)
    }

    async sendCancelAll(targetPublicKey: Uint8Array | string, marketAdminPublicKey: Uint8Array | string) {
        const signedTransaction = await TenexUtils.buildSignedTransaction(
            /* request */ TenexTransaction.buildCancelAllRequest(targetPublicKey, marketAdminPublicKey),
            /* senderPrivKey */ this.privateKey,
            /* recentBlockDigest */ undefined,
            /* fee */ undefined,
            /* client */ this.client
        );
        console.log('Sending cancel all tranasction')
        await this.client.submitTransaction(signedTransaction)
        // console.log('result=', result)
        // FermiUtils.checkSubmissionResult(result)
    }
}

export const delay = ms => new Promise(res => setTimeout(res, ms));