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
import { delay, getJsonRpcUrl, SDKAdapter } from "./utils"
import localConfig from "./localConfig.json"
// import { DeploymentBuilder } from "../deploy_futures/deployFuturesMarket";

// EXTERNAL
import { FermiClient, FermiTypes, FermiUtils, FermiAccount } from 'fermi-js-sdk'
import { TenexUtils, TenexTransaction, TenexClient } from 'tenex-js-sdk';

async function main() {
    const coinbase_resp = await fetch(exchanges["coinbase"].replace("[SYMBOL]", "ALGO"));
    const coinbase_data = await coinbase_resp.json().then(x => x.data.rates);

    while (true) {
        const symbolToAssetId = localConfig['symbolToAssetId'];
        const authorities = Object.keys(config["authorities"])
        const deploymentAuthority = config['authorities'][authorities[localConfig.deploymentAuthority]]
        const client = new TenexClient(getJsonRpcUrl(deploymentAuthority))
        
    
        const marketPlaces = await client.getFuturesMarketPlaces();
        let marketPlace = marketPlaces[0];
        console.log('Fetching Markets from First Marketplace');
        const markets = await client.getFuturesMarkets(marketPlace.admin);
        console.log('Markets: ', markets);

        console.log('Fetching Market Admin User Data from Marketplace');
        const marketAdminData = await client.getUserMarketplaceInfo(
          marketPlace.admin,
          marketPlace.admin
        );

        const deployerPrivateKey = FermiUtils.hexToBytes(deploymentAuthority.private_key)
        const deployerPublicKey = await FermiAccount.getPublicKey(deployerPrivateKey);
    
        console.log('Market Admin Data: ', marketAdminData);
        const user_market_info = marketAdminData.user_market_info;
        for (var market_info of user_market_info){
            console.log("market_info: ", market_info);

            for (var order of market_info.orders) {
                console.log("Order: ", order);
            }
        }
        
        const adapter = new SDKAdapter(deployerPrivateKey, client)
        
        
        let assetIdsPrices: number[][] = [];

        for (var symbol of Object.keys(symbolToAssetId)) {
            if (symbol == "FRMI") {
                assetIdsPrices.push([symbolToAssetId[symbol], 1]);
                continue
            } else if (symbol == "USDC") {
                continue
            }
            const coinbase_price = 1./coinbase_data[symbol];
    
            const kucoin_resp = await fetch(exchanges["kucoin"].replace("[SYMBOL]", symbol));
            //@ts-ignore
            const kucoin_data = await kucoin_resp.json().then(x => x.data);
            const kucoin_price = Number(kucoin_data['price']);
    
            const binance_resp = await fetch(exchanges["binance"].replace("[SYMBOL]", symbol));
            const binance_data = await binance_resp.json();
            //@ts-ignore
            const bitcoin_price = Number(binance_data['price']);
    
            let avg_price = (coinbase_price + kucoin_price + bitcoin_price) / 3.;
            avg_price = parseInt(String(avg_price));
            assetIdsPrices.push([symbolToAssetId[symbol], avg_price])

            await adapter.sendCancelAll(deployerPublicKey, FermiUtils.hexToBytes(marketPlace.admin)); // TODO - the multiple handling of marketPlace admin is idiotic
            for (var delta = 1; delta < 11; delta++) {
                await adapter.sendFuturesLimitOrderRequest(
                /* baseAssetId */ symbolToAssetId[symbol],
                /* quoteAssetId */ 1,
                /* side */ 1,
                /* price */ avg_price-delta,
                /* quantity */ delta,
                /* admin */ FermiUtils.hexToBytes(marketPlace.admin)
              );

              await adapter.sendFuturesLimitOrderRequest(
                /* baseAssetId */ symbolToAssetId[symbol],
                /* quoteAssetId */ 1,
                /* side */ 2,
                /* price */ avg_price+delta,
                /* quantity */ delta,
                /* admin */ FermiUtils.hexToBytes(marketPlace.admin)
              );
            }

            break
        }
        
        // break
        // const coinbase_resp = await fetch(exchanges["coinbase"].replace("[SYMBOL]", "ALGO"));
        // //@ts-ignore
        // const coinbase_data = await coinbase_resp.json().then(x => x.data.rates);
    
        // let assetIdsPrices: number[][] = [];
        // for (var symbol of Object.keys(symbolToAssetId)) {
        //     console.log("symbol: ", symbol)
        //     if (symbol == "FRMI") {
        //         assetIdsPrices.push([symbolToAssetId[symbol], 1]);
        //         continue
        //     } else if (symbol == "USDC") {
        //         continue
        //     }
        //     const coinbase_price = 1./coinbase_data[symbol];
    
        //     const kucoin_resp = await fetch(exchanges["kucoin"].replace("[SYMBOL]", symbol));
        //     //@ts-ignore
        //     const kucoin_data = await kucoin_resp.json().then(x => x.data);
        //     const kucoin_price = Number(kucoin_data['price']);
    
        //     const binance_resp = await fetch(exchanges["binance"].replace("[SYMBOL]", symbol));
        //     const binance_data = await binance_resp.json();
        //     //@ts-ignore
        //     const bitcoin_price = Number(binance_data['price']);
    
        //     const avg_price = (coinbase_price + kucoin_price + bitcoin_price) / 3.;
        //     assetIdsPrices.push([symbolToAssetId[symbol], parseInt(String(avg_price))])
        // }
        // console.log("latest assetIdsPrices = ", assetIdsPrices);
        // await deployer.sendUpdatePricesRequest(/* latestPrices */ assetIdsPrices);
    }
}

main();
