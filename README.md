# ❄️ Frosty Functions ⚡

Run your off-chain workloads on-chain
- Decentralized
- Flexible & Fast (WASM)
- Super Powers (HTTP calls, EVM interaction, ...)

## Goal

The goal of this project is to make it trivial for any developer to run arbitrary
code on the Internet Computer without any learning curve. Furthermore, it should be trivial to call that code from any EVM smart contract and to receive callbacks.

## Proof of concept

This proof of concept demonstrates:

- Running wasmi inside an ICP canister
- Setting instruction limits and resuming execution if desired (can be used for charging gas)
- Compiling Assembly Script into wasm
- Host functions to extend functionality

## Technical Architecture

Inspired by [icp-evm-coprocessor-starter](https://github.com/letmejustputthishere/icp-evm-coprocessor-starter)

## Local testing

### Using anvil and remix

* Start `anvil`
* Launch https://remix.ethereum.org/
  * Compile Bridge.sol
  * Connect to localhost using Custom - External Http Provider
  * Deploy contract with e.g.
    * Owner 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (anvil account[1])
    * MinPaymentWei 100
  * Invoke functions with e.g.
    * FunctionId "0x2f044fb7f581f6deeace9cd91d68500fcd439b3fe82729771c8b4385522ad576"
    * Data 0xdeadbeef
    * Value 100000
* You can view logged events with `cast logs --rpc-url http://localhost:854
5 --address 0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9 --from-block 0 --to-block latest` (replace address with contract address)


