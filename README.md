# ❄️ Frosty Functions

> [!NOTE]
> This is very much a work-in-progress project. The main technical challenges are solved though, so getting close to an MVP!

## Overview

Frosty Functions allow you to execute TypeScript¹ on the [Internet Computer](https://internetcomputer.org/what-is-the-ic) blockchain without having to deploy your own canister. They can be invoked from EVM smart contracts and control assets programmatically across multiple chains using [Threshold signatures](https://internetcomputer.org/docs/references/t-sigs-how-it-works).

- **TypeScript-like:** Frosty Functions are written in [AssemblyScript](https://www.assemblyscript.org/), a variant of TypeScript that compiles into WASM. Developers can write code in a familiar language directly in the Frosty web app without having to learn Rust or Motoko or install anything on their machine. The WASM binary is interpreted by the canister using [wasmi](https://github.com/wasmi-labs/wasmi). 
- **Serverless and canister-less:** Developers should be able to just focus on their code, rather than having to manage cansiters and cycles. In the future, Frosty will use verifiable builds and non-upgradeable cansiters in order to still provide the same security and decentralization guarantees that you get from a native cansiter. You will also be able to self host  your own "Frosty cloud” easily.
- **EVM-interop:** Frosty Functions can be invoked from any smart contract or EOA via a Bridge contract. Currently this is deployed on Arbitrum One and Arbitrum Sepolia (testnet), but more EVM chains as well as Bitcoin and Solana will be supported in the future. Functions have full control over a wallet held by distributed private keys using the Internet Computer’s Chain Fusion technology, so that they can invoke arbitrary smart contracts on other chains.

## Vision

The larger vision for **❄️ Frosty** is to become the developer platform that makes building decentralized web3 apps as easy as building web2 apps with Firebase.

Primary design goals:
1. **Seamless developer experience:** Frosty does not invent any new cryptography, but aims to make it accessible to a much larger developer base. For example, Frosty Functions doesn't require any installation or new tokens, it works out of the box with a simple MetaMask wallet.
1. **Scalable:** The architecture is designed to be able to scale horizontally in the future. See [building planet-scale apps on ICP](https://forum.dfinity.org/t/building-planet-scale-apps-on-icp/59346). This is why Frosty Functions are intentionally stateless (Wallets do hold state, but my plan is to split wallet and executor cansiters in the future).
1. **Cross-chain:** While the Internet Computer makes for a great execution layer due to its horizontally scalable architecture, the future will probably be cross-chain in some way. Frosty aims to hide that complexity though from developers, e.g. making it easy to handle micropayments on various chains.
1. **Decentralized & Secure:** Many "web3" apps rely heavily on centralized off-chain actors. Frosty aim to find secure and decentralized solutions for all components.

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
