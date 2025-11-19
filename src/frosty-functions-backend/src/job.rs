use evm_rpc_types::{Hex32, Nat256};

use crate::chain::Address;

/// Request for executing a function. Currently these are created from EVM logs,
/// but in the future they could also come from other sources such as other chains,
/// recursive invocations etc.
#[derive(Debug)]
pub struct JobRequest {
    /// Chain ID in CAIP-2 format (e.g., "eip155:1" for Ethereum mainnet)
    pub chain_id: String,
    /// Block hash this log was found in
    pub block_hash: Option<Hex32>,
    /// Block number this log was found in
    pub block_number: Option<Nat256>,
    /// Transaction hash that emitted this log
    pub transaction_hash: Option<Hex32>,

    /// On-chain job id. This ID is unique on-chain, but can be duplicate due
    /// to re-orgs.
    pub on_chain_id: Option<Nat256>,
    /// Caller address formatted as string (e.g. 0x...)
    pub caller: Address,
    /// SHA-256 of the wasm of the function that should be executed.
    pub function_hash: [u8; 32],
    /// Arbitrary payload passed to the function
    pub data: Vec<u8>,
    /// Gas payment forwarded with the call in the native currency of the calling chain.
    pub gas_payment: Nat256,
}

/// Job with metadata and execution state.
#[derive(Debug)]
pub struct Job {
    pub request: JobRequest,
}
