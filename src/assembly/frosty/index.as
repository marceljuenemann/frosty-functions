import { __calldata, __example_async_host_function } from "./internal";
import { SharedPromise } from "frosty/internal/async";
import { Promise } from "frosty/promise";

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

export function example_async(): Promise<string> {
  console.log("example_async called");
  let promise = new SharedPromise();
  __example_async_host_function(promise.ref);
  return promise.map<string>((value: i32) => {
    console.log(`example_async: Host function returned with value: ${value}`);
    return value.toString();
  });
}
