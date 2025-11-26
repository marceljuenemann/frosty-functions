import { CALLDATA_SIZE, calldata, example_async } from "./frosty";
import { Callback, Promise } from "./frosty/promise";

class SimpleCallback implements Callback<string> {
  onFulfilled(value: string): void {
    console.log(`SimpleCallback fulfilled with value: ${value}`);
  }

  onRejected(reason: Error): void {
    console.log(`SimpleCallback rejected with reason: ${reason.message}`);
  }
}

export function main(): void {
  console.log("Welcome to main()");

  console.log(`Calldata size is: ${CALLDATA_SIZE}`);
  console.log(`Calldata is: ${calldata()}`);



  let res = new Promise<string>();
  res.resolve("42");
  console.log(`Resolved promise value is: ${res}`);

  let rej = new Promise<i32>();
  rej.reject(new Error("Rejected promise"));
  console.log(`Resolved promise value is: ${rej}`);


  res.addCallback(new SimpleCallback());
  let prom = res.map<i32>((value) => {
    console.log(`then called with ${value}`);
    return value.length;
  });

  prom.then((value) => {
    console.log(`Final length value: ${value}`);
  });

  /*
  res.then<string>((value) => value + " and ")
    .then<string>((value) => value + " more")
    .then<void>((value) => {
      console.log(`Final resolved value: ${value}`);
    });
*/

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

  console.log("main() finished");
}
