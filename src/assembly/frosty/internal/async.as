import { Promise } from "frosty/promise";

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

/**
 * A Promise that can be resolved by the host.
 */
export class SharedPromise extends Promise<i32> {
  public readonly id: i32;

  constructor() {
    super();
    this.id = PROMISE_REGISTRY.register(this);
  }
}

export function resolveSharedPromise(id: i32, value: i32): void {
  PROMISE_REGISTRY.retrieve(id).resolve(value);
}

export function rejectSharedPromise(id: i32, value: i32): void {
  // TODO: Load error message from shared buffer.
  PROMISE_REGISTRY.retrieve(id).reject(new Error(`Promise rejected with value: ${value}`));
}
