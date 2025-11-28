/**
 * Frosty runtime that is included with every build.
 */

import { rejectSharedPromise, resolveSharedPromise } from "frosty/internal/async";

/**
 * Called by the host to resolve a promise.
 */
export function __frosty_resolve(promiseId: i32, dataSize: i32): void {
  resolveSharedPromise(promiseId, dataSize);
}

/**
 * Called by the host to reject a promise.
 */
export function __frosty_reject(promiseId: i32, dataSize: i32): void {
  rejectSharedPromise(promiseId, dataSize);
} 
