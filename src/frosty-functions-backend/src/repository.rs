use alloy::primitives::{FixedBytes, keccak256};
use candid::{CandidType};
use serde::Deserialize;

use crate::storage::{get_function, store_function};

pub type FunctionId = Vec<u8>; // Keccak256 hash (32 bytes) of the function binary.

#[derive(Clone, CandidType, Debug, Deserialize)]
pub struct FunctionDefinition {
    binary: Vec<u8>,
    source: String,
    compiler: String,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct FunctionState {
    pub definition: FunctionDefinition,
    pub hash: FunctionId,
    pub deployed_at: u64,  // Timestamp in nanoseconds
    pub is_verified: bool,
}

#[derive(CandidType, Debug)]
pub enum DeployResult {
    Success(FunctionId),
    Duplicate(FunctionId),
    Error(String),
}

pub fn deploy_function(definition: FunctionDefinition) -> DeployResult {
    let id = keccak256(&definition.binary).to_vec();
    if get_function(id.clone()).is_some() {
        return DeployResult::Duplicate(id);
    }

    // TODO: Run a simulation to verify integrity before storing.

    store_function(id.clone(), FunctionState {
        definition,
        hash: id.clone(),
        deployed_at: ic_cdk::api::time(),
        is_verified: false,
    });

    DeployResult::Success(id)
}
