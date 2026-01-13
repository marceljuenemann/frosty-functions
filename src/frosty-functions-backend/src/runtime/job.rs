use std::ops::Sub;

use candid::{CandidType, Nat};
use evm_rpc_types::{Hex32, Nat256};
use serde::{Deserialize, Serialize};

use crate::{chain::{Address, Chain}, repository::FunctionId};

/// Request for executing a function. Currently these are created from EVM logs,
/// but in the future they could also come from other sources such as other chains,
/// recursive invocations etc.
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct JobRequest {
    /// Chain that this job request originates from.
    pub chain: Chain,
    /// Block hash this log was found in
    pub block_hash: Option<Hex32>,
    /// Block number this log was found in
    pub block_number: Option<u64>,
    /// Transaction hash that emitted this log
    pub transaction_hash: Option<Hex32>,

    /// On-chain job id. This ID is unique on-chain, but can be duplicate due
    /// to re-orgs.
    pub on_chain_id: Option<Nat256>,
    /// Caller address.
    pub caller: Address,
    /// SHA-256 of the wasm of the function that should be executed.
    pub function_hash: FunctionId,
    /// Arbitrary payload passed to the function
    pub data: Vec<u8>,
    /// Gas payment forwarded with the call in the native currency of the calling chain.
    pub gas_payment: Nat256,
}

/// Job with metadata and execution state.
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct Job {
    // The request that created this job.
    pub request: JobRequest,
    // Current status of the job.
    pub status: JobStatus,
    // Timestamp when the job was created (Unix nanoseconds).
    pub created_at: u64,
    /// Execution is split into a series of commits. This field
    /// contains the IDs of all commits for this job made so far.
    pub commit_ids: Vec<u64>,
    // Fees charged for the execution of this job. Excludes gas_used.
    pub fees: u64,
    // Gas used for transactions on the calling chain (e.g. depositGas).
    pub gas: u64,
}

impl Job {
    pub fn new(request: JobRequest) -> Self {
        Self {
            request,
            status: JobStatus::Pending,
            created_at: ic_cdk::api::time(),
            commit_ids: Vec::new(),
            fees: 0,
            gas: 0,
        }
    }

    pub fn total_cost(&self) -> u64 {
        self.fees + self.gas
    }

    pub fn remaining_gas(&self) -> Nat {
        return self.request.gas_payment.as_ref().clone().sub(self.total_cost());
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub enum JobStatus {
    /// Job was added to the queue, but not yet processed.
    Pending,
    /// Job is currently being executed.
    Executing,
    /// Job is waiting for a timer to wake it up again.
    Waiting,
    /// Job completed without errors.
    Completed,
    /// Job execution failed.
    Failed(String)  // Change to proper error type.
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Commit {
    pub timestamp: u64,
    pub title: String,
    pub logs: Vec<LogEntry>,
    pub instructions: u64,  // Host instructions used.
    pub fees: u64,          // Fees charged for this commit.
}

/**
 * Log entry with different log levels.
 */
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct LogEntry {
    pub level: LogType,
    pub message: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum LogType {
    System,
    Default,
}

/*
type FailureReason = variant {
  FunctionNotFound;      // No function with the given ID
  InvalidModule;         // Wasm module is invalid or malformed
  OutOfGas;              // Ran out of gas during execution
  UncaughtException;     // Uncaught exception in user code
  SystemError;           // Something that should not happen.
};

*/
