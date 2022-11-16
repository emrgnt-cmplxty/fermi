// Execute this script with the command: yarn execute-trade
// This script will fund a taker authority and execute a trade into the futures market
// This script assumes that the market has been deployed via yarn deploy-futures
// And that a valid network has been deployed with associated data saved to protonet.json

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

class TakerBuilder {
  public takerPrivateKey: Uint8Array;
  public funderPrivateKey: Uint8Array;
  public client: FermiClient;

  constructor(
    takerPrivateKey: Uint8Array,
    funderPrivateKey: Uint8Array,
    client: FermiClient
  ) {
    this.takerPrivateKey = takerPrivateKey;
    this.funderPrivateKey = funderPrivateKey;
    this.client = client;
  }

  async fundTakerAccount(quantity: number) {
    const takerPublicKey = await FermiAccount.getPublicKey(
      this.takerPrivateKey
    );

    const signedTransaction = await TenexUtils.buildSignedTransaction(
      /* request */ TenexTransaction.buildPaymentRequest(
        /* receiver */ takerPublicKey,
        /* assetId */ 1,
        quantity
      ),
      /* senderPrivKey */ this.funderPrivateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending fund account tranasction');
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
      /* senderPrivKey */ this.takerPrivateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending account deposit request');
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
      /* senderPrivKey */ this.takerPrivateKey,
      /* recentBlockDigest */ undefined,
      /* fee */ undefined,
      /* client */ this.client
    );
    console.log('Sending futures limit order request');
    const result: FermiTypes.QueriedTransaction =
      await this.client.sendAndConfirmTransaction(signedTransaction);
    console.log('result=', result);
    FermiUtils.checkSubmissionResult(result);
  }
}

async function main() {
  console.log('config=', config);
  const authorities = Object.keys(config['authorities']);

  const futuresAuthority =
    config['authorities'][authorities[localConfig.deploymentAuthority]];
  const futuresPrivateKey = FermiUtils.hexToBytes(futuresAuthority.private_key);
  const futuresPublicKey = await FermiAccount.getPublicKey(futuresPrivateKey);

  const takerAuthority =
    config['authorities'][authorities[localConfig.takerAuthority]];
  const client = new FermiClient(getJsonRpcUrl(takerAuthority));
  const takerPrivateKey = FermiUtils.hexToBytes(takerAuthority.private_key);

  console.log('Funding another authority and taking the available liquidity!');

  const marketTaker = new TakerBuilder(
    takerPrivateKey,
    futuresPrivateKey,
    client
  );

  await marketTaker.fundTakerAccount(/* quantity */ 1_000_000);

  await marketTaker.sendAccountDepositRequest(
    /* quantity */ 1_000_000,
    /* marketAdmin */ futuresPublicKey
  );

  // Take the opposite side of the order specified in deployFuturesMarket
  await marketTaker.sendFuturesLimitOrderRequest(
    /* baseAssetId */ 0,
    /* quoteAssetId */ 1,
    /* side */ 2,
    /* price */ 100,
    /* quantity */ 100,
    /* admin */ futuresPublicKey
  );

  console.log('Successfully Traded!');
}
main();
