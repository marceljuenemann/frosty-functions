use alloy::primitives::{FixedBytes, keccak256};
use candid::{CandidType};
use serde::Deserialize;

use crate::stable::store_function;

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
    // For now, just print the function definition details.
    // TODO: Return duplicate.
    ic_cdk::println!("Deploying function:");
    ic_cdk::println!("Source: {}", definition.source.len());
    ic_cdk::println!("Compiler: {}", definition.compiler);
    ic_cdk::println!("Binary size: {} bytes", definition.binary.len());

    let hash = keccak256(&definition.binary);
    ic_cdk::println!("Binary hash: 0x{}", hex::encode(hash));

    store_function(hash.to_vec(), FunctionState {
        definition,
        hash: hash.to_vec(),
        deployed_at: ic_cdk::api::time(),
        is_verified: false,
    });

    DeployResult::Success(hash.to_vec())
}
