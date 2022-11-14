// Execute this script with the command: yarn execute-trade
// This script will fund a taker authority and execute a trade into the futures market
// This script assumes that the market has been deployed via yarn deploy-futures
// And that a valid network has been deployed with associated data saved to protonet.json

// LOCAL
import config from '../../configs/protonet.json';
import localConfig from './localConfig.json';
import { getJsonRpcUrl } from './utils';

// EXTERNAL
import assert from 'assert';
import {
  FermiClient,
  FermiTypes,
  FermiUtils,
  FermiAccount,
} from 'fermi-js-sdk';
import { TenexClient } from 'tenex-js-sdk';

const DEFAULT_JSONRPC_ADDRESS = 'http://localhost:3006';

async function main() {
  console.log('config=', config);
  const authorities = Object.keys(config['authorities']);

  let client = new TenexClient.default(DEFAULT_JSONRPC_ADDRESS);
  console.log('Fetching Market Places');
  const marketPlaces = await client.getFuturesMarketPlaces();

  console.log('marketPlaces = ', marketPlaces);

  let marketPlace = marketPlaces[0];
  console.log('Fetching Markets from First Marketplace');
  const markets = await client.getFuturesMarkets(marketPlace.admin);
  console.log('Markets: ', markets);

  console.log('Fetching Market Admin User Data from Marketplace');
  const marketAdminData = await client.getUserMarketplaceInfo(
    marketPlace.admin,
    marketPlace.admin
  );
  console.log('Market Admin Data: ', marketAdminData);

  console.log('Fetching Market Taker User Data from Marketplace');
  // Is there a cleaner way to fetch teh appropriate public key for the FermiUtils input?
  const takerAuthority =
    config['authorities'][authorities[localConfig.takerAuthority]];
  const takerPublicKey = await FermiAccount.getPublicKey(
    FermiUtils.hexToBytes(takerAuthority.private_key)
  );
  const marketTakerData = await client.getUserMarketplaceInfo(
    marketPlace.admin,
    FermiUtils.bytesToHex(takerPublicKey)
  );
  console.log('Market Taker Data: ', marketTakerData);

  console.log('Successfully Traded!');
}
main();
