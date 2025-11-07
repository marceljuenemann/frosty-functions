
use std::collections::HashMap;
use crate::job::Job;

/// Stores all state related to a specific blockchain.
pub struct ChainState {
    // The chain ID using CAIP-2 format.
    pub chain_id: String,

    // The bridge contract address on this chain (e.g., EVM 0x... address).
    pub bridge_address: String,

    // The highest block number that has been synced.
    pub synced_block_number: u64,

    // A mapping of job IDs to their corresponding Job structs.
    pub jobs: HashMap<u64, Job>,

    // A queue of job IDs that still need to be processed.
    pub job_queue: Vec<u64>,
}

impl ChainState {
    pub fn new(chain_id: String, bridge_address: String) -> Self {
        Self {
            chain_id,
            bridge_address,
            synced_block_number: 0,
            jobs: HashMap::new(),
            job_queue: Vec::new(),
        }
    }
}
