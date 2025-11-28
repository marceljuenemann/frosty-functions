import { __example_async_host_function } from "./internal";
import { SharedPromise } from "frosty/internal/async";
import { Promise } from "frosty/promise";

/**
 * The data payload passed when the Frosty Function was invoked.
 */
@lazy
export const CALLDATA = ((): Uint8Array => {
  // Note that the compiler won't strip this out, even if CALLDATA
  // is never used. So we should prefer functions over constants for
  // constants less likely to be used.
  let buffer = new ArrayBuffer(CALLDATA_SIZE);
  __calldata(changetype<i32>(buffer));
  return Uint8Array.wrap(buffer);
})();

@external("❄️", "CALLDATA_SIZE")
declare const CALLDATA_SIZE: i32;

@external("❄️", "calldata")
declare function __calldata(buffer_ptr: i32): void;


/**
 * Job ID generated on the block chain that invoked this contract.
 * 
 * Note that this ID is only unique in the context of the calling chain.
 * Even then, you might observe multiple invocations with the same Job ID
 * in case blocks get re-orged. This function might return -1 in the future
 * when invoked off-chain.
 */
@lazy
export const JOB_ID = __on_chain_id();

@external("❄️", "on_chain_id")
declare function __on_chain_id(): u64;

/**
 * Returns the calldata that was passed with the function invocation.
 */

export function example_async(): Promise<string> {
  console.log("example_async called");
  let promise = new SharedPromise();
  __example_async_host_function(promise.ref);
  return promise.map<string>((value: i32) => {
    console.log(`example_async: Host function returned with value: ${value}`);
    return value.toString();
  });
}
