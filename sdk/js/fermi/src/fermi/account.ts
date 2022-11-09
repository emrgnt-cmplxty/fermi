// IMPORTS
import { getPublicKey, sign, utils } from '@noble/ed25519'
import * as FermiUtils from './utils'
export { getPublicKey, sign }

export default class FermiAccount {
  public privateKey: Uint8Array
  public publicKey: Uint8Array
  public publicAddress: string

  constructor(privateKey: Uint8Array, publicKey: Uint8Array) {
    this.privateKey = privateKey
    this.publicKey = publicKey
    this.publicAddress = FermiUtils.bytesToHex(this.publicKey)
  }
}

export async function generateAccount(): Promise<FermiAccount> {
  const privateKey = await utils.randomPrivateKey()
  const publicKey = await getPublicKey(privateKey)
  return new FermiAccount(privateKey, publicKey)
}