/**
 * This file declares the host functions exposed by Frosty. These should not be
 * used directly and are not considered part of the stable API.
 */

@external("❄️", "calldata")
export declare function __calldata(buffer_ptr: i32): void;

@external("❄️", "example_host_function")
export declare function example_host_function(): i64;

@external("❄️", "example_async_host_function")
export declare function example_async_host_function(callback: i32): void;
