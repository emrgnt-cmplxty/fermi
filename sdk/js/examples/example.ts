// IMPORTS
import { TenexClient } from '../dist';
import { buildPaymentRequest } from '../dist/utils/bankController'
import { getPublicKey } from '@noble/ed25519';


// TODO is there a cleaner way to represent this test data - https://github.com/gdexorg/gdex/issues/180
export const exampleData = {
  defaultReceiver: Uint8Array.from([
    180, 53, 40, 38, 78, 42, 79, 75, 150, 70, 135, 195, 161, 68, 26, 164, 208, 64, 71, 114, 114, 30, 151, 49, 35, 175,
    6, 175, 179, 30, 165, 232,
  ]),
  defaultSender: Uint8Array.from([
    34, 234, 101, 99, 203, 223, 74, 40, 136, 149, 156, 173, 137, 140, 109, 54, 20, 32, 83, 35, 141, 186, 181, 238, 152,
    209, 101, 200, 83, 74, 11, 71,
  ]),
}

// UTILITIES

const DEFAULT_RELAYER_ADDRESS = 'localhost:3014';
const DEFAULT_TRANSACTION_SUBMITTER_ADDRESS = "localhost:3005";

async function test() {
    // GETTING INFORMATION, TESTING WITH PAYMENT TRANSACTION FIRST
    // Example of how to retrieve everything out of a transaction
    let tenexClient = new TenexClient(DEFAULT_RELAYER_ADDRESS, DEFAULT_TRANSACTION_SUBMITTER_ADDRESS);

    const receiver = await getPublicKey(exampleData.defaultReceiver)
    const paymentRequest = buildPaymentRequest(receiver, 0, 100)
    const signedTransaction = await tenexClient.buildSignedTransaction(
      /* request */ paymentRequest,
      /* senderPrivKey */ exampleData.defaultSender
    )
    const result = await tenexClient.sendAndConfirmTransaction(signedTransaction);
    console.log("Result: ", result)
}
test();
