# ❄️ Frosty Functions

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

## Technical Architecture

The underlying technology is very similiar to [icp-evm-coprocessor-starter](https://github.com/letmejustputthishere/icp-evm-coprocessor-starter). Frosty Functions just takes this one step further by building a Coprocessor-as-a-Service.

Life of a Frosty Function:
1. Developers can write the code directly in the web app without the need for any installation or accounts or tokens. The code is compiled client-side using AssemblyScript and the Frosty standard library, which provides a high-level interface to host functions.
1. Developers can "simulate" a function execution through the web app right away. This is executed on the canister as a _query_ call without any side effects
1. Developers can then deploy their function into the Frosty canister. The hash of the uploaded WASM binary is used to identify the function going forward
1. Anybody can now invoke the function through the [Bridge contract](contracts/Bridge.sol) deployed on a supported chain
1. Currently, the `index_block` cansiter method needs to be invoked manually to index the event (the Frontend takes care of this, but for smart contract invocations this needs to be done off-chain). This step will be fully automated in the future.
1. The cansiter verifies the event through a [HTTP Outcall](https://internetcomputer.org/https-outcalls/) to an RPC service. Currently, only Alchemy is used, but in the future the plan is to query multiple providers to reduce centralization risks.
1. The function invocation is now added to a job queue. In the future, the actual execution will be delegated to an available execution canister (or a new one will be spawned if needed).
1. The function may request control of a wallet for the caller using `Wallet.forCaller()`. This wallet is shared between all Frosty Functions, but unique depending on the caller that invoked the function. That allows a smart contract or user to use different functions to manage the same assets, but also means they need to trust the function they are invoking. In the future, different wallets will be available by specifying a derivation path (also wallets shared between all callers of the same function).

## Security Considerations

> [!WARNING]
> The Frosty Functions contract is currently sill upgradable, meaning anybody with the dev keys could deploy new code that signs arbitrary messages. You should therefore only use Frosty Functions for development purposes.

In addition to the above, Frosty Functions comes with the following trust assumptions:

1. You trust the subnet of the Internet Computer that Frosty Functions runs on. This is currently <a href="https://dashboard.internetcomputer.org/network/subnets/fuqsr-in2lc-zbcjj-ydmcw-pzq7h-4xm2z-pto4i-dcyee-5z4rz-x63ji-nae">fuqsr-in2lc-zbcjj-ydmcw-pzq7h-4xm2z-pto4i-dcyee-5z4rz-x63ji-nae</a>. If 5 of the 13 node providers conspire, they could arbitrarily change the execution of Frosty.
2. You trust the Internet Computer's <a href="https://dashboard.internetcomputer.org/network/subnets/pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae">Fiduciary subnet</a>, which consists of 34 nodes that execute the threshold signing.
3. Events of the bridge contract are currently only fetched via Alchemy, so Alchemy could create fake events on behalf of contracts by changing their RPC response. #1 will expand this to requiring consensus between multiple RPC providers.
4. You trust the code in this repository.


## Features

- [x] Invoke from EVM
  - [x] Local EVM
  - [x] Arbitrum Sepolia
  - [ ] Arbitrum One
  - [ ] ETH Sepolia
  - [ ] ETH mainnet
- [x] AssemblyScript support
- [x] Frontend (written in Angular)
  - [x] Simulate functions
  - [x] Deployment
  - [x] Invoke functions
  - [x] View logs
- [x] Async functions with Promises
- [x] console.log support
- [x] Verifiable Random Function (VRF)
- [ ] Gas payments
  - [x] Deposit gas into wallet
  - [x] Charge for instructions
  - [x] Charge for outcalls
  - [ ] Gas refunds
- [ ] Wallet and Provider interfaces
  - [x] address()
  - [x] signMessage()
  - [ ] eth_sendTransaction
  - [ ] eth_call
  - [ ] eth_getLogs
 - [ ] Memory limit
 - [ ] Server-side builds (canister?)
 - [ ] Automated indexing

 - [ ] ABI & Solidity support (compile-time imports & transformations)
 - [ ] Long-running calls (no-op async call when reaching ICP's 40B instruction limit)
 - [ ] fetch (HTTP Outcalls)
 - [ ] Timers (delay() Promise)
 - [ ] Canister calls
 - [ ] Invoke other Frosty Functions (including recursively) 
 - [ ] x402 support (invoke via REST)
 - [ ] Cross-chain support
   - [ ] Bitcoin
   - [ ] Solana
   - [ ] Helpers for token swaps
  

## Contract deployments

Latest version of the contract

* Arbitrum Sepolia (testnet):
  * 0xcAcbb4E46F2a68e3d178Fb98dCaCe59d12d54CBc for owner 0xda824f554c42ecd28a74a037c70fa0b5bf447bb0 (localhost)
* Arbitrum One: https://arbiscan.io/address/0xe712a7e50aba019a6d225584583b09c4265b037b

## Canister deployments

* Frontend: https://vayms-xiaaa-aaaao-qmb6q-cai.icp0.io/
* Backend: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=n6va3-cyaaa-aaaao-qk6pq-cai

* Example function: https://vayms-xiaaa-aaaao-qmb6q-cai.icp0.io/functions/0x0dbfc27a4145ff75aa1c3bb153331e5a88d077dd175444b5bd52aa6d8b3c411b
  * Example execution: https://vayms-xiaaa-aaaao-qmb6q-cai.icp0.io/chains/eip155:42161/jobs/1

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


## License

Unless otherwise specified, all code in this repository is licensed under the AGPL to ensure that any improvements are shared back with the community. If you deploy a modified version of Frosty Functions, you must provide the source code to your users. See full [LICENSE](./LICENSE)

Copyright ©2025 Marcel Juenemann

Copyright ©2025 HTTP 403 Limited

Frosty Functions is being developed and run by HTTP 403 Limited.
