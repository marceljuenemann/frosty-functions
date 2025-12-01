import { CALLDATA, JOB_ID } from "frosty";
import { CALLING_CHAIN_NAME, CALLING_CHAIN_ID, Wallet } from "frosty/evm";
import { ArrayBufferPromise } from "frosty/promise";
import { verifiableRandomness } from "frosty/random";
import { toHexString } from "frosty/util";

export function main(): void {
  console.log(`Invoked from ${CALLING_CHAIN_NAME} (Chain ID: ${CALLING_CHAIN_ID})`);
  console.log(`Calldata is: ${toHexString(CALLDATA)}`);
  console.log(`Job ID is: ${JOB_ID}`);

  // Frosty automatically creates an Ethereum Wallet for the caller that invoked the
  // function. The private key is split across the nodes of the Internet Computer and
  // signs messages using Threshold ECDSA. Note that any Frosty Function can manage the
  // the `forCaller()` wallet as long as that caller invokes the function.
  const wallet = Wallet.forCaller();
//  console.log(`Caller address: ${CALLER_ADDRESS}`);
  console.log(`Caller Frosty Wallet address: ${wallet.address}`);




  //examples.randomness();

  /*
  evm.callback(new ArrayBuffer(0), 1300).then((data: ArrayBuffer) => {
    console.log(`EVM callback completed with data: ${toHexString(Uint8Array.wrap(data))}`);
  });
  */



  /**

  const wallet = Wallet.forCaller();  // Synchronous, fetch before execution (only once).
  console.log(`Caller address: ${CALLER_ADDRESS}`);
  console.log(`Caller Frosty Wallet address: ${wallet.address()}`);

  wallet.signMessage("Hello, World!").then((signature) => {
    console.log(`Signed message: ${toHexString(signature)}`);
  });

  // TODO: Check for minimum amount
  // TODO: Generate a random number
  // TODO: Convert the random amount to WETH

  // Deposits are deducted from the gas of the current call. Make sure to
  // leave enough gas for the function execution to complete! (More helper
  // methods to manage gas will be added in the future.)
  wallet.depositGwei(10000).then((wallet) => {


  
  
  })

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
