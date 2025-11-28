import { Promise } from "./promise";
import { SharedPromise } from "./internal/async";

/**
 * Returns 32 verifiably random bytes.
 * 
 * Refer to the Internet Computer documentation for details.
 * @see https://internetcomputer.org/docs/building-apps/network-features/randomness
 */
// TODO: Convert to array
export function verifiableRandomness(): Promise<ArrayBuffer> {
  console.log("verifiableRandomness called");
  let promise = new SharedPromise();
  ic_raw_rand(promise.id);
  return promise.map<ArrayBuffer>((value: i32) => {
    console.log(`Promise resolved with: ${value}`);
    return new ArrayBuffer(32); // TODO: Replace with actual random bytes
  });
}

@external("❄️", "ic_raw_rand")
declare function ic_raw_rand(promise_id: i32): void;
