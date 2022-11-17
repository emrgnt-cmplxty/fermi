// Execute this script with the command: yarn deploy-futures
// This script will deploy the futures market and launch a single test trade
// Deployment is made from the perspective of the deployment authority specified in localConfig.json
// This script assumes that a valid network has been deployed with associated data saved to protonet.json

// IMPORTS

// LOCAL
import config from '../../configs/protonet.json';
import localConfig from './localConfig.json';
import { getJsonRpcUrl, SDKAdapter } from './utils';

// EXTERNAL
import {
  FermiClient,
  FermiTypes,
  FermiUtils,
  FermiAccount,
} from 'fermi-js-sdk';
import { TenexTransaction, TenexUtils } from 'tenex-js-sdk';

async function main() {
  console.log('config=', config);
  const symbolToAssetId = localConfig['symbolToAssetId'];
  const authorities = Object.keys(config['authorities']);
  const deploymentAuthority =
    config['authorities'][authorities[localConfig.deploymentAuthority]];
  console.log('deploymentAuthority=', deploymentAuthority);

  const client = new FermiClient(getJsonRpcUrl(deploymentAuthority));

  const deployerPrivateKey = FermiUtils.hexToBytes(
    deploymentAuthority.private_key
  );
  const deployerPublicKey = await FermiAccount.getPublicKey(deployerPrivateKey);

  const deployer = new SDKAdapter(deployerPrivateKey, client);
  console.log('Starting Deployment Now...');

  await deployer.sendCreateAsset(/* dummy */ 0); // primary asset
  const usdcAssetId =  Number(symbolToAssetId["USDC"]);
  console.log("usdcAssetId=")
  await deployer.sendCreateAsset(/* dummy */ usdcAssetId); // usdc asset

  await deployer.sendCreateMarketplaceRequest(/* quoteAssetId */ usdcAssetId);

  let assetIdsPrices: number[][] = [];
  for (var symbol of Object.keys(symbolToAssetId)) {
    if (symbol == "USDC") {
      continue
    }
    const assetId = Number(symbolToAssetId[symbol])
    await deployer.sendCreateAsset(/* dummy */ assetId);
    await deployer.sendCreateMarketRequest(/* baseAssetId */ assetId);
    assetIdsPrices.push([assetId, 1_000_000]);
  };

  console.log('assetIdsPrices=', assetIdsPrices);
  // // additional assets
  await deployer.sendUpdatePricesRequest(/* latestPrices */ assetIdsPrices);

  await deployer.sendAccountDepositRequest(
    /* quantity */ 1_000_000_000_000,
    /* marketAdmin */ deployerPublicKey
  );

  await deployer.sendFuturesLimitOrderRequest(
    /* baseAssetId */ 2,
    /* quoteAssetId */ 1,
    /* side */ 1,
    /* price */ 1_000,
    /* quantity */ 1_000,
    /* admin */ deployerPublicKey
  );

  console.log('Successfully Deployed And Tested!');
}
main();
