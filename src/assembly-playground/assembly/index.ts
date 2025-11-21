// import { ic } from "./system";

// Global array to hold callback references, forcing them into the function table
let callbackRegistry: (() => void)[] = [];

export function main(): void {
  console.log("Welcome to main()");

  let x = example_host_function();
  example_async( named_callback );
  example_async((): void => {
    console.log(`Async callback invoked! Host function returned`);
    example_host_function();
  });

  console.log("main() finished");
}

function named_callback(): void {
  console.log("named_callback invoked");
}

function example_async(callback: () => void): void {
  // Store the callback in the array to ensure it's compiled and in the table
  callbackRegistry.push(callback);
  
  // Get the function index from the last added callback
  let funcIndex = changetype<i32>(callback);
  example_async_host_function(funcIndex);
}

@external("❄️", "example_host_function")
declare function example_host_function(): i64;

@external("❄️", "example_async_host_function")
declare function example_async_host_function(callback: i32): void;
