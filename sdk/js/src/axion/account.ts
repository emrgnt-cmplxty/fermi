// IMPORTS
import { getPublicKey, sign, utils } from '@noble/ed25519'

export { getPublicKey, sign, utils }

export default class AxionAccount {
  public privateKey: Uint8Array
  public publicKey: Uint8Array
  public publicAddress: string

  constructor(privateKey: Uint8Array, publicKey: Uint8Array) {
    this.privateKey = privateKey
    this.publicKey = publicKey
    this.publicAddress = utils.bytesToHex(this.publicKey)
  }
}

export async function generateAccount(): Promise<AxionAccount> {
  const privateKey = await utils.randomPrivateKey()
  const publicKey = await getPublicKey(privateKey)
  return new AxionAccount(privateKey, publicKey)
}
