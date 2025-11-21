/**
 * Registry for callback functions. Functions in a registry can be retrieved again
 * with a numeric ID and won't be garbage collected while in the registry.
 */
class CallbackRegistry<T> {
  private map: Map<i32, T> = new Map<i32, T>();
  private nextId: i32 = 0;

  public register(callback: T): i32 {
    const id = this.nextId++;
    this.map.set(id, callback);
    return id;
  }
  
  public retrieve(id: i32): T {
    if (!this.map.has(id)) {
      throw new Error(`FROSTY: No callback found with id: ${id}`);
    }
    return this.map.get(id);
  }
}

/**
 * Low-level promise for host <=> module communication.
 */
export interface HostPromise {
  resolve(bufferLength: i32): void;
  reject(bufferLength: i32): void;
}

/**
 * Global promise registry for async host function callbacks.
 * 
 * By keeping references to callbacks here, we ensure they won't be garbage collected.
 */
export const PROMISE_REGISTRY = new CallbackRegistry<HostPromise>();
