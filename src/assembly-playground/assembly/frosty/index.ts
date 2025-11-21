import { __calldata, __example_async_host_function } from "./internal";

/**
 * Size of the calldata passed into the function in bytes.
 */
@external("❄️", "CALLDATA_SIZE")
export declare const CALLDATA_SIZE: i32;

/**
 * Returns the calldata that was passed with the function invocation.
 */
export function calldata(): Uint8Array {
  let buffer = new ArrayBuffer(CALLDATA_SIZE);
  __calldata(changetype<i32>(buffer));
  return Uint8Array.wrap(buffer);
}


const callbackRegistry: (() => void)[] = [];

export function example_async(callback: () => void): void {
  console.log("example_async called");
  callbackRegistry.push(callback);
  const callbackId = callbackRegistry.length - 1;
  __example_async_host_function(callbackId);
}
