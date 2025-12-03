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
  private value: Box<T> | null = null;
  private reason: Error | null = null;
  private callbacks: Array<Callback<T>> = [];

  resolve(value: T): void {
    if (this.state === PromiseState.Pending) {
      this.state = PromiseState.Fulfilled;
      this.value = new Box(value);
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
  private addCallback(callback: Callback<T>): void {
    switch (this.state) {
      case PromiseState.Fulfilled:
        callback.onFulfilled(this.value!.inner);
        break;

      case PromiseState.Rejected:
        callback.onRejected(this.reason!);
        break;

      case PromiseState.Pending:
        this.callbacks.push(callback);
        break;
    }
  }

  // TODO: Add methods for catching rejected promises: catching, mapError, onError, finally?

  /**
   * Transforms the Promise value using a callback that has access to additional context.
   * Only invoked if the Promise is fulfilled successfully.
   */  
  mapWith<C, U>(context: C, onSuccess: (context: C, value: T) => U): Promise<U> {
    let callback = new ClosureCallback<C, T, U>(context, onSuccess, null);
    this.addCallback(callback);
    return callback.nextPromise;
  }

  map<U>(onSuccess : (value: T) => U): Promise<U> {
    // Reuse mapWith by passing the callback function as context.
    return this.mapWith<(value: T) => U, U>(onSuccess, (context, value) => context(value));
  }

  then(onSuccess: (value: T) => void): Promise<T> {
    // Reuse mapWith by passing the callback function as context.
    this.mapWith<(value: T) => void, Done>(onSuccess, (onSuccess, value) => {
      onSuccess(value);
      return DONE;
    });
    return this;
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

class Box<T> {
  constructor(public readonly inner: T) {}
}

export class Done {
  static DONE: Done = new Done();

  private constructor() {}
}
export const DONE = Done.DONE;

interface Callback<T> {
  onFulfilled(value: T): void;
  onRejected(reason: Error): void;
}

/**
 * A callback that can hold additional context to work around the lack of 
 * closures.
 * 
 * This callback always creates a new Promise of type U whose value is generated
 * by the onFulfilledHandler if the original Promise is resolved. If onRejectedHandler
 * is provided, it must recover to type U as well. Otherwise errors are propagated.
 */
class ClosureCallback<C, T, U> implements Callback<T> {
  public readonly nextPromise: Promise<U> = new Promise<U>();

  constructor(
    private readonly context: C,
    private readonly onFulfilledHandler: (context: C, value: T) => U,
    private readonly onRejectedHandler: ((reason: Error, context: C) => U) | null
  ) {}

  onFulfilled(value: T): void {
    this.nextPromise.resolve(this.onFulfilledHandler(this.context, value));
  }

  onRejected(reason: Error): void {
    if (this.onRejectedHandler) {
      // Recover to a fulfilled state.
      this.nextPromise.resolve(this.onRejectedHandler(reason, this.context));
    } else {
      // Propagate rejection if no handler is provided.
      this.nextPromise.reject(reason);
    }
  }
}
