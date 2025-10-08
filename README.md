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
