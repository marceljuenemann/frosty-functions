import { Promise, ArrayBufferPromise } from "frosty/promise";

/**
 * A Promise that can be resolved by the host.
 */
export class SharedPromise extends ArrayBufferPromise {
  public readonly id: i32;

  constructor() {
    super();
    this.id = PROMISE_REGISTRY.register(this);
  }
}

/**
 * We keep promises in a registry so they won't be garbage collected
 * and can be retrieved again when the callback is invoked from the host.
 */
class Registry<T> {
  private map: Map<i32, T> = new Map<i32, T>();
  private nextId: i32 = 0;

  public register(callback: T): i32 {
    const id = this.nextId++;
    this.map.set(id, callback);
    return id;
  }
  
  public retrieve(id: i32): T {
    let value = this.map.get(id);
    this.map.delete(id);
    return value;
  }
}

const PROMISE_REGISTRY = new Registry<SharedPromise>();

export function resolveSharedPromise(id: i32, dataSize: i32): void {
  const buffer = new ArrayBuffer(dataSize);
  copy_shared_buffer(changetype<i32>(buffer));
  PROMISE_REGISTRY.retrieve(id).resolve(buffer);
}

export function rejectSharedPromise(id: i32, dataSize: i32): void {
  // TODO: Load error message from shared buffer.
  PROMISE_REGISTRY.retrieve(id).reject(new Error(`Promise rejected with value: ${dataSize}`));
}

@external("❄️", "copy_shared_buffer")
declare function copy_shared_buffer(destinationPtr: i32): void;
