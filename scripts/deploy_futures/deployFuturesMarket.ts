// Execute this script with the command: yarn deploy-futures
// This script will deploy the futures market and launch a single test trade
// Deployment is made from the perspective of the deployment authority specified in localConfig.json
// This script assumes that a valid network has been deployed with associated data saved to protonet.json

// IMPORTS

// LOCAL
import config from '../../configs/protonet.json';
import localConfig from './localConfig.json';
import { getJsonRpcUrl } from './utils';

// EXTERNAL
import {
  FermiClient,
  FermiTypes,
  FermiUtils,
  FermiAccount,
} from 'fermi-js-sdk';
import { TenexTransaction, TenexUtils } from 'tenex-js-sdk';

class DeploymentBuilder {
  public privateKey: Uint8Array;
  public client: FermiClient;

  constructor(privateKey: Uint8Array, client: FermiClient) {
    this.privateKey = privateKey;
    this.client = client;
  }

  async sendCreateAsset(dummy: number) {
    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildCreateAssetRequest(dummy),
      /* senderPrivKey */ this.privateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending asset creation tranasction');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }

  async sendCreateMarketplaceRequest(quoteAssetId: number) {
    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildCreateMarketplaceRequest(
        quoteAssetId
      ),
      /* senderPrivKey */ this.privateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending create marketplace tranasction');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }

  async sendCreateMarketRequest(baseAssetId: number) {
    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildCreateMarketRequest(baseAssetId),
      /* senderPrivKey */ this.privateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending create market tranasction');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }

  async sendUpdatePricesRequest(latestPrices: number[]) {
    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildUpdatePricesRequest(latestPrices),
      /* senderPrivKey */ this.privateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending update prices tranasction');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }

  async sendAccountDepositRequest(quantity: number, marketAdmin: Uint8Array) {
    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildAccountDepositRequest(
        quantity,
        marketAdmin
      ),
      /* senderPrivKey */ this.privateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending create market tranasction');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }

  async sendFuturesLimitOrderRequest(
    baseAssetId: number,
    quoteAssetId: number,
    side: number,
    price: number,
    quantity: number,
    marketAdmin: Uint8Array
  ) {
    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildFuturesLimitOrderRequest(
        baseAssetId,
        quoteAssetId,
        side,
        price,
        quantity,
        marketAdmin
      ),
      /* senderPrivKey */ this.privateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending create market tranasction');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }
}

async function main() {
  console.log('config=', config);
  const authorities = Object.keys(config['authorities']);
  const deploymentAuthority =
    config['authorities'][authorities[localConfig.deploymentAuthority]];
  console.log('deploymentAuthority=', deploymentAuthority);

  const client = new FermiClient(getJsonRpcUrl(deploymentAuthority));

  const deployerPrivateKey = FermiUtils.hexToBytes(
    deploymentAuthority.private_key
  );
  const deployerPublicKey = await FermiAccount.getPublicKey(deployerPrivateKey);

  const deployer = new DeploymentBuilder(deployerPrivateKey, client);
  console.log('Starting Deployment Now...');

  await deployer.sendCreateAsset(/* dummy */ 0);
  await deployer.sendCreateAsset(/* dummy */ 1);

  await deployer.sendCreateMarketplaceRequest(/* quoteAssetId */ 1);

  await deployer.sendCreateMarketRequest(/* baseAssetId */ 0);

  await deployer.sendUpdatePricesRequest(/* latestPrices */ [1_000_000]);

  await deployer.sendAccountDepositRequest(
    /* quantity */ 1_000_000,
    /* marketAdmin */ deployerPublicKey
  );

  await deployer.sendFuturesLimitOrderRequest(
    /* baseAssetId */ 0,
    /* quoteAssetId */ 1,
    /* side */ 1,
    /* price */ 1_000,
    /* quantity */ 1_000,
    /* admin */ deployerPublicKey
  );

  console.log('Successfully Deployed And Tested!');
}
main();
