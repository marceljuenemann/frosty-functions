import { CALLDATA, JOB_ID } from "frosty";
import * as evm from "frosty/evm";
import { ArrayBufferPromise } from "frosty/promise";
import { verifiableRandomness } from "frosty/random";
import { toHexString } from "frosty/util";

export  function main(): void {
  console.log(`Invoked from ${evm.CALLING_CHAIN_NAME} (Chain ID: ${evm.CALLING_CHAIN_ID})`);
  console.log(`Job ID is: ${JOB_ID}`);
  console.log(`Calldata is: ${toHexString(CALLDATA)}`);

  //examples.randomness();
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
