import { Promise } from "../promise";

class Registry<T> {
  private map: Map<i32, T> = new Map<i32, T>();
  private nextId: i32 = 0;

  public register(callback: T): i32 {
    const id = this.nextId++;
    this.map.set(id, callback);
    return id;
  }
  
  public retrieve(id: i32): T {
    return this.map.get(id);
  }
}

const PROMISE_REGISTRY = new Registry<SharedPromise>();

/**
 * A Promise that can be resolved by the host.
 */
export class SharedPromise extends Promise<i32> {
  public readonly reference: i32;

  constructor() {
    super();
    this.reference = PROMISE_REGISTRY.register(this);
  }
}

export function resolveSharedPromise(reference: i32, value: i32): void {
  PROMISE_REGISTRY.retrieve(reference).resolve(value);
}

export function rejectSharedPromise(reference: i32, value: i32): void {
  // TODO: Load error message from shared buffer.
  PROMISE_REGISTRY.retrieve(reference).reject(new Error(`Promise rejected with value: ${value}`));
}
