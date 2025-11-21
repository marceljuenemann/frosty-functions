import { CALLDATA_SIZE, calldata, example_async } from "./frosty";

export function main(): void {
  console.log("Welcome to main()");

  console.log(`Calldata size is: ${CALLDATA_SIZE}`);
  console.log(`Calldata is: ${calldata()}`);

  example_async(() => {
    console.log(`Async callback invoked! Host function returned`);
  });

  console.log("main() finished");
}

function named_callback(): void {
  console.log("named_callback invoked");
}
