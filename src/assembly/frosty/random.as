import { Promise } from "./promise";
import { SharedPromise } from "./internal/async";

/**
 * Returns 32 verifiably random bytes.
 * 
 * Refer to the Internet Computer documentation for details.
 * @see https://internetcomputer.org/docs/building-apps/network-features/randomness
 */
export function verifiableRandomness(): Promise<ArrayBuffer> {
  let promise = new SharedPromise();
  ic_raw_rand(promise.id);
  return promise;
}

@external("❄️", "ic_raw_rand")
declare function ic_raw_rand(promise_id: i32): void;
