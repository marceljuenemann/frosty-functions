import { CALLDATA, JOB_ID } from "frosty";
import * as evm from "frosty/evm";

export  function main(): void {
  console.log(`Invoked from ${evm.CALLING_CHAIN_NAME} (Chain ID: ${evm.CALLING_CHAIN_ID})`);
  console.log(`Job ID is: ${JOB_ID}`);
  console.log(`Calldata is: ${CALLDATA}`);
}

/*
  example_async()
    .map<string>((value) => "Mapped value: " + value.toString())
    .then(
      value => {
        console.log(`Final output: ${value}`)
      },
      err => {
        console.log(`example_async failed with error: ${err.message}`);
      }
    );
*/
