export enum PromiseState {
  Pending = 0,
  Fulfilled = 1,
  Rejected = 2
}

/**
 * Adaptation of Promise for AssemblyScript. Given that closures are not supported,
 * the API differrs quite a bit, but emphasize is put on leaving the main user facing
 * API (.then) the same.
 */
// TODO: This is work in progress. Need to add tests and handle all edge cases.
export class Promise<T> {
  private state: PromiseState = PromiseState.Pending;
  private value: T = changetype<T>(0);
  private reason: Error = changetype<Error>(0);
  private callbacks: Array<Callback<T>> = [];

  resolve(value: T): void {
    if (this.state === PromiseState.Pending) {
      this.state = PromiseState.Fulfilled;
      this.value = value;
      for (let i = 0; i < this.callbacks.length; i++) {
        this.callbacks[i].onFulfilled(value);
      }
      this.callbacks = [];
    }
  }

  reject(reason: Error): void {
    if (this.state === PromiseState.Pending) {
      this.state = PromiseState.Rejected;
      this.reason = reason;
      for (let i = 0; i < this.callbacks.length; i++) {
        this.callbacks[i].onRejected(reason);
      }
      this.callbacks = [];
    }
  }
  
  /**
   * Add a callback that is invoked exactly once.
   */
  addCallback(callback: Callback<T>): void {
    switch (this.state) {
      case PromiseState.Fulfilled:
        callback.onFulfilled(this.value);
        break;

      case PromiseState.Rejected:
        callback.onRejected(this.reason);
        break;

      case PromiseState.Pending:
        this.callbacks.push(callback);
        break;
    }
  }

  map<U>(onFulfilled: (value: T) => U, onError: ((reason: Error) => U) | null = null): Promise<U> {
    let callback = new MappingCallback<T, U>(onFulfilled, onError);
    this.addCallback(callback);
    return callback.nextPromise;
  }

  then(onFulfilled: ((value: T) => void) | null, onError: ((reason: Error) => void) | null = null): void {
    this.addCallback(new DelegatingCallback<T>(onFulfilled, onError));
  }

  // TODO: Implement catch and finally.

  isPending(): bool {
    return this.state === PromiseState.Pending;
  }

  isFulfilled(): bool {
    return this.state === PromiseState.Fulfilled;
  }

  isRejected(): bool {
    return this.state === PromiseState.Rejected;
  }

  toString(): string {
    // TODO: Improve
    return `[Promise state=${this.state}, value=${this.value}]`;
  }

  static resolve<T>(value: T): Promise<T> {
    let promise = new Promise<T>();
    promise.resolve(value);
    return promise;
  }

  static reject<T>(reason: Error): Promise<T> {
    let promise = new Promise<T>();
    promise.reject(reason);
    return promise;
  }
}

export interface Callback<T> {
  onFulfilled(value: T): void;
  onRejected(reason: Error): void;
}

/**
 * Callback that delegates to provided handlers.
 */
class DelegatingCallback<T> implements Callback<T> {
  constructor(
    private readonly onFulfilledHandler: ((value: T) => void) | null,
    private readonly onRejectedHandler: ((reason: Error) => void) | null
  ) {}
  
  onFulfilled(value: T): void {
    if (this.onFulfilledHandler) {
      this.onFulfilledHandler(value);
    }
  }

  onRejected(reason: Error): void {
    if (this.onRejectedHandler) {
      this.onRejectedHandler(reason);
    }
  }
}

/**
 * Callback that maps the result to another Promise.
 */
class MappingCallback<T, U> implements Callback<T> {
  public readonly nextPromise: Promise<U> = new Promise<U>();

  constructor(
    private readonly onFulfilledHandler: (value: T) => U,
    private readonly onRejectedHandler: ((reason: Error) => U) | null
  ) {}

  onFulfilled(value: T): void {
    this.nextPromise.resolve(this.onFulfilledHandler(value));
  }

  onRejected(reason: Error): void {
    if (this.onRejectedHandler) {
      // Recover to a fulfilled state.
      this.nextPromise.resolve(this.onRejectedHandler(reason));
    } else {
      // Propagate rejection if no handler is provided.
      this.nextPromise.reject(reason);
    }
  }
}

export class ArrayBufferPromise extends Promise<ArrayBuffer> {

  asUint8Array(): Promise<Uint8Array> {
    return this.map<Uint8Array>((buffer: ArrayBuffer) => Uint8Array.wrap(buffer));
  }
  
  asUint16Array(): Promise<Uint16Array> {
    return this.map<Uint16Array>((buffer: ArrayBuffer) => Uint16Array.wrap(buffer));
  }
  
  asUint32Array(): Promise<Uint32Array> {
    return this.map<Uint32Array>((buffer: ArrayBuffer) => Uint32Array.wrap(buffer));
  }

  asUint64Array(): Promise<Uint64Array> {
    return this.map<Uint64Array>((buffer: ArrayBuffer) => Uint64Array.wrap(buffer));
  }

  asInt8Array(): Promise<Int8Array> {
    return this.map<Int8Array>((buffer: ArrayBuffer) => Int8Array.wrap(buffer));
  }
  
  asInt16Array(): Promise<Int16Array> {
    return this.map<Int16Array>((buffer: ArrayBuffer) => Int16Array.wrap(buffer));
  }
  
  asInt32Array(): Promise<Int32Array> {
    return this.map<Int32Array>((buffer: ArrayBuffer) => Int32Array.wrap(buffer));
  }

  asInt64Array(): Promise<Int64Array> {
    return this.map<Int64Array>((buffer: ArrayBuffer) => Int64Array.wrap(buffer));
  }

  asFloat32Array(): Promise<Float32Array> {
    return this.map<Float32Array>((buffer: ArrayBuffer) => Float32Array.wrap(buffer));
  }
  
  asFloat64Array(): Promise<Float64Array> {
    return this.map<Float64Array>((buffer: ArrayBuffer) => Float64Array.wrap(buffer));
  }  
}
