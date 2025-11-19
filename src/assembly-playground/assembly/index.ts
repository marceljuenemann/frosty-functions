// import { ic } from "./system";

const CON = 32 * 34;

// console.log(`CON is ${CON}`);

const TIME = example_host_function();

export function add(a: i32, b: i32): i32 {
  return a + b + i32(Math.abs(a - b));
}

export function main(): void {
  let x = example_host_function();
  /*
  const currentTime = ic.time();
  const randomNum = ic.randomInt(100);
  console.log(`Large fibonacci is ${fib(165500)}`);
  return (currentTime << 32) + randomNum;
  */
}

function fib(n: i32): i32 {
  var a = 0, b = 1
  if (n > 0) {
    while (--n) {
      let t = a + b
      a = b
      b = t
    }
    return b
  }
  return a
}

@external("env", "example_host_function")
declare function example_host_function(): i64;
