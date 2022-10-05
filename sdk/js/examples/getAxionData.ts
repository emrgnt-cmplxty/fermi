// IMPORTS

// INTERNAL
import { AxionClient } from '../src/axion'

const DEFAULT_JSONRPC_ADDRESS = 'http://localhost:3006'
async function main() {
  let client = new AxionClient(DEFAULT_JSONRPC_ADDRESS)
  console.log('fetching now...')

  let latestBlockInfo = await client.getLatestBlockInfo()
  // An example response follows below:
  //
  //
  // latestBlockInfo =  {
  //   validator_system_epoch_time_in_micros: 1664914635824250,
  //   block_number: 131168,
  //   block_id: '0x23a4001a11c639a4130363935e1064d13d3f777f6a2f3496113bf1f5d7bece44'
  // }
  //
  //
  //
  console.log('latestBlockInfo = ', latestBlockInfo)

  let firstBlockInfo = await client.getBlockInfo(/* blockNumber */ 1)
  // An example response follows below:
  //
  //
  // latestBlockInfo =  {
  //   validator_system_epoch_time_in_micros: 1664911300461308,
  //   block_number: 1,
  //   block_id: '0x7e7366d15e8d91b3ba3928cef72568f0f31751ec600bc30ada81d39061822af6'
  // }
  //
  //
  //
  console.log('firstBlockInfo = ', firstBlockInfo)

  let firstBlock = await client.getBlock(/* blockNumber */ 1)
  // An example response follows below:
  //
  //
  // firstBlock =  {
  //   transactions: [
  //     {
  //       executed_transaction: [Object],
  //       transactionId: '0xb2d6265783d3084e804fb58cbafbea7ef24cf5bd7c7e6548b2b726404a436e04'
  //     },
  //     {
  //       executed_transaction: [Object],
  //       transactionId: '0x1247b383228332835c9b209e946d39a6438d8b804e77e0f7447a2031fecdedfc'
  //     },
  //     ...
  //   ],
  //   block_id: '0x13237fce5e19c51d822bc5fc5dc20275a8416b163cb4683cf0a54ec812f0db58'
  // }
  //
  //
  //
  console.log('firstBlock = ', firstBlock)
}
main()
