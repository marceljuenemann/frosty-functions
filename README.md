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

## Local development

### Testing using anvil and remix

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
* Using the candid backend container, you can now sync and execute the jobs
  * `add_chain("eip155:31337", "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0")` (replace with bridge address)
  * `sync_chain("eip155:31337")` (31337 will automatically use localhost as RPC Service)
  * `get_queue("eip155:31337")`
  * `execute_job("eip155:31337", job_id)`

### Regenerate candid interface

See instructions [here](https://internetcomputer.org/docs/building-apps/developer-tools/cdks/rust/generating-candid#option-1-automatic-generation-using-generate-did-crate). There seems to be some issue with finding the right WASM file though, so I currently use `generate-candid.sh` as a workaround.

### Assembly Script Playground

Until the Frontend is up and running, the WASM needs to be compiled directly with the Assembly
Script compiler. Run `npm run asbuild` in `src/assembly-playground`.

## Contract deployments

Latest version of the contract

* Arbitrum Sepolia (testnet): https://sepolia.arbiscan.io/address/0xe712a7e50aba019a6d225584583b09c4265b037b
* Arbitrum One: https://arbiscan.io/address/0xe712a7e50aba019a6d225584583b09c4265b037b
