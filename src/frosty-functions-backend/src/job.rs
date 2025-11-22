use candid::CandidType;
use evm_rpc_types::{Hex32, Nat256};
use serde::{Deserialize, Serialize};

use crate::chain::{Address, Chain};

/// Request for executing a function. Currently these are created from EVM logs,
/// but in the future they could also come from other sources such as other chains,
/// recursive invocations etc.
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct JobRequest {
    /// Chain that this job request originates from.
    pub chain: Chain,
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
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct Job {
    pub request: JobRequest,
    // TODO: Add status, timestamps, logs, gas used etc.
}


/*
type FailureReason = variant {
  FunctionNotFound;      // No function with the given ID
  InvalidModule;         // Wasm module is invalid or malformed
  OutOfGas;              // Ran out of gas during execution
  UncaughtException;     // Uncaught exception in user code
  SystemError;           // Something that should not happen.
};

type JobStatus = variant {
  // Job was added to the queue, but not yet processed.
  Pending;

  // Job is currently being executed.
  Processing;
  
  // Job completed without errors.
  Completed;

  // Job execution failed.
  Failed: record { error: FailureReason };
};

*/