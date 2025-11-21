import { __calldata } from "./internal";

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
