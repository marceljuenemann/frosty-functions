import { CALLDATA, JOB_ID } from "frosty";

export function main(): void {
  console.log("Welcome to main()");

  console.log(`Job ID is: ${JOB_ID}`);
  console.log(`Calldata is: ${CALLDATA}`);

/*
  example_async()
    .map<string>((value) => "Mapped value: " + value.toString())
    .then(
      value => {
        console.log(`Final output: ${value}`)
      },
      err => {
        console.log(`example_async failed with error: ${err.message}`);
      }
    );
*/
}
