import { CALLDATA, JOB_ID } from "frosty";
import { CALLING_CHAIN_NAME, CALLING_CHAIN_ID, EthWallet } from "frosty/evm";
import { ArrayBufferPromise } from "frosty/promise";
import { verifiableRandomness } from "frosty/random";
import { hex } from "frosty/util";

export function main(): void {
  console.log(`Invoked from ${CALLING_CHAIN_NAME} (Chain ID: ${CALLING_CHAIN_ID})`);
  console.log(`Calldata is: ${hex.encode(CALLDATA)}`);
  console.log(`Job ID is: ${JOB_ID}`);

  // Frosty automatically creates an Ethereum Wallet for the caller that invoked the
  // function. The private key is split across the nodes of the Internet Computer and
  // signs messages using Threshold ECDSA. Note that any Frosty Function can manage the
  // the `forCaller()` wallet as long as that caller agreed to invoke the function.
  const wallet = EthWallet.forCaller();
//  console.log(`Caller address: ${CALLER_ADDRESS}`);
  console.log(`Caller Frosty Wallet address: ${wallet.address()}`);

  const randomness: ArrayBufferPromise = verifiableRandomness()
  randomness.then((rand) => console.log(`Converted to Uint8Array: ${Uint8Array.wrap(rand)}`))

  /*
  // You can use wallet.signMessage to sign arbitrary EIP-191 messages.
  wallet.signMessage(String.UTF8.encode("Hello, World!")).then((signature) => {
    console.log(`Signature: ${hex.encode(signature)}`);
  });

  // You can transfer gas that you sent to the Frosty function into the wallet.
  // However, make sure to leave enough gas for the function execution to complete!
  // Note that this actually results in an Ethereum transaction with the usual gas
  // costs, which will be deducted from your gas account. Alternatively, you can also
  // send funds directly to wallet.address() outside of Frosty.
  wallet.depositGas(10000).then((txHash) => {
    console.log(`Transferred gas into Wallet.forCaller(). Tx hash: ${hex.encode(txHash)}`);
  });
  */

  //examples.randomness();

  /**
  // TODO: Check for minimum amount
  // TODO: Generate a random number
  // TODO: Convert the random amount to WETH
  */

}


/****************************************/
/********* MORE EXAMPLES BELOW **********/ 
/****************************************/

namespace examples {

  /**
   * verifiableRandomness returns an ArrayBufferPromise that can be
   * converted to various typed arrays.
   */
  function randomness(): void {
    const randomness: ArrayBufferPromise = verifiableRandomness()
    randomness.asUint8Array().then((rand) => console.log(`Converted to Uint8Array: ${rand}`))
    randomness.asUint16Array().then((rand) => console.log(`Converted to Uint16Array: ${rand}`))
    randomness.asUint32Array().then((rand) => console.log(`Converted to Uint32Array: ${rand}`))
    randomness.asUint64Array().then((rand) => console.log(`Converted to Uint64Array: ${rand}`))
    randomness.asInt8Array().then((rand) => console.log(`Converted to Int8Array: ${rand}`))
    randomness.asInt16Array().then((rand) => console.log(`Converted to Int16Array: ${rand}`))
    randomness.asInt32Array().then((rand) => console.log(`Converted to Int32Array: ${rand}`))
    randomness.asInt64Array().then((rand) => console.log(`Converted to Int64Array: ${rand}`))
    randomness.asFloat32Array().then((rand) => console.log(`Converted to Float32Array: ${rand}`))
    randomness.asFloat64Array().then((rand) => console.log(`Converted to Float64Array: ${rand}`))  
  }
}
