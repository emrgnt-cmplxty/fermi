// IMPORTS
import { generateAccount } from '../src/axion/account'

async function main() {
  const account = await generateAccount()
  // An example account follows below:
  //
  //
  // AxionAccount {
  //   PrivateKey: Uint8Array(32) [
  //     188, 201,  83, 169, 151, 254,  84,
  //     171, 248, 170, 139,  81, 112, 141,
  //      55, 171, 208,  82, 186,  81, 188,
  //      72, 117,  84,  28,  31, 235, 213,
  //     182, 122, 198, 158
  //   ],
  //   PublicKey: Uint8Array(32) [
  //      11, 120, 193, 167,  17,  43, 168,  93,
  //     192, 175, 223,   2, 111, 135, 211, 250,
  //     101, 214, 158,  57, 132, 103,  31, 130,
  //      98, 244, 222, 254, 236,  35, 113,  41
  //   ],
  //   publicAddress: '0b78c1a7112ba85dc0afdf026f87d3fa65d69e3984671f8262f4defeec237129'
  // }
  //
  //
  //
  console.log(account)
}
main()
