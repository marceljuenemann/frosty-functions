import { CALLDATA, JOB_ID } from "frosty";
import * as evm from "frosty/evm";
import { verifiableRandomness } from "frosty/random";
import { toHexString } from "frosty/util";

export  function main(): void {
  console.log(`Invoked from ${evm.CALLING_CHAIN_NAME} (Chain ID: ${evm.CALLING_CHAIN_ID})`);
  console.log(`Job ID is: ${JOB_ID}`);
  console.log(`Calldata is: ${toHexString(CALLDATA)}`);

  verifiableRandomness().then((randomness) => {
    console.log("Received verifiable randomness: " + randomness.toString());
  });
}
