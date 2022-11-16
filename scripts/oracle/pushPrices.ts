// Execute this script with the command: yarn deploy-futures
// This script will deploy the futures market and launch a single test trade
// Deployment is made from the perspective of the deployment authority specified in localConfig.json
// This script assumes that a valid network has been deployed with associated data saved to protonet.json

// IMPORTS

// LOCAL
import config from "../../configs/protonet.json"
import exchanges from "./exchanges.json"
import fetch from 'node-fetch';
// cleanup this script imoport
import { getJsonRpcUrl } from "../deploy_futures/utils"
import localConfig from "../deploy_futures/localConfig.json"
// import { DeploymentBuilder } from "../deploy_futures/deployFuturesMarket";

// EXTERNAL
import { FermiClient, FermiTypes, FermiUtils, FermiAccount } from 'fermi-sdk'
import { TenexTransaction, TenexUtils } from 'tenex-sdk'

class PricePusher {
  
        public privateKey: Uint8Array
        public client: FermiClient
      
        constructor(privateKey: Uint8Array, client: FermiClient) {
          this.privateKey = privateKey
          this.client = client
        }

        async sendUpdatePricesRequest(latestPrices: number[]) {
            const signedTransaction = await TenexUtils.buildSignedTransaction(
                /* request */ TenexTransaction.buildUpdatePricesRequest(latestPrices),
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
    }

const symbols = ["BTC", "ETH"]
async function main() {
    const authorities = Object.keys(config["authorities"])
    const deploymentAuthority = config['authorities'][authorities[localConfig.deploymentAuthority]]
    const client = new FermiClient(getJsonRpcUrl(deploymentAuthority))
    
    const deployerPrivateKey = FermiUtils.hexToBytes(deploymentAuthority.private_key)
    const deployer = new PricePusher(deployerPrivateKey, client)

    const coinbase_resp = await fetch(exchanges["coinbase"].replace("[SYMBOL]", "ALGO"));
    const coinbase_data = await coinbase_resp.json().then(x => x.data.rates);

    for (var symbol of symbols) {
        const coinbase_price = 1./coinbase_data[symbol];

        const kucoin_resp = await fetch(exchanges["kucoin"].replace("[SYMBOL]", symbol));
        const kucoin_data = await kucoin_resp.json().then(x => x.data);
        const kucoin_price = Number(kucoin_data['price']);

        const binance_resp = await fetch(exchanges["binance"].replace("[SYMBOL]", symbol));
        const binance_data = await binance_resp.json();
        const bitcoin_price = Number(binance_data['price']);

        const avg_price = (coinbase_price + kucoin_price + bitcoin_price) / 3.;
        await deployer.sendUpdatePricesRequest(/* latestPrices */ [parseInt(String(avg_price))]);
    }

}

main();
