import { __calldata, __example_async_host_function } from "./internal";
import { HostPromise, PROMISE_REGISTRY } from "./internal/async";

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

class ExampleAsyncCallback implements HostPromise {
  constructor(private callback: () => void) {}

  resolve(bufferLength: i32): void {
    console.log(`example_async: Promise resolved with bufferLength: ${bufferLength}`);
    this.callback();
  }

  reject(bufferLength: i32): void {
    console.log(`example_async: Promise rejected with bufferLength: ${bufferLength}`);
  }

  // TODO: Could be part of a base class?
  register(): i32 {
    return PROMISE_REGISTRY.register(this);
  }
}

export function example_async(callback: () => void): void {
  console.log("example_async called");
  __example_async_host_function(new ExampleAsyncCallback(callback).register());
}
