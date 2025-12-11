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
  * Deploy contract with
    * Owner set to the canister EMV address. Can be retrieved with `get_evm_address()` canister method
      * Probably 0xda824f554c42ecd28a74a037c70fa0b5bf447bb0
    * MinPaymentWei 100
  * Update contract address in the code (currently hardcoded)
    * evm.rs
    * signer.ts
    * Address should be 0x5FbDB2315678afecb367f032d93F642f64180aa3 if you used the values from above
  * Invoke functions with e.g.
    * FunctionId "0x2f044fb7f581f6deeace9cd91d68500fcd439b3fe82729771c8b4385522ad576"
    * Data 0xdeadbeef
    * Value 100000
* You can view logged events with `cast logs --rpc-url http://localhost:854
5 --address 0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9 --from-block 0 --to-block latest` (replace address with contract address)
* Using the candid backend container, you can now sync and execute the jobs
  * `sync_chain`
  * `get_queue`
  * `execute_job`
* You can check balances with `cast balance 0xA7AC21127fE04FbE9502AEeFDd55204CF51f2C25 --rpc-url ht
tp://localhost:8545 --ether`
  
### Regenerate candid interface

See instructions [here](https://internetcomputer.org/docs/building-apps/developer-tools/cdks/rust/generating-candid#option-1-automatic-generation-using-generate-did-crate). There seems to be some issue with finding the right WASM file though, so I currently use `generate-candid.sh` as a workaround.

### Assembly Script Playground

Until the Frontend is up and running, the WASM needs to be compiled directly with the Assembly
Script compiler. Run `npm run asbuild` in `src/assembly-playground`.

## Contract deployments

Latest version of the contract

* Arbitrum Sepolia (testnet):
  * 0xcAcbb4E46F2a68e3d178Fb98dCaCe59d12d54CBc for owner 0xda824f554c42ecd28a74a037c70fa0b5bf447bb0 (localhost)
* Arbitrum One: https://arbiscan.io/address/0xe712a7e50aba019a6d225584583b09c4265b037b

## Canister deployments

* Backend: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=n6va3-cyaaa-aaaao-qk6pq-cai
