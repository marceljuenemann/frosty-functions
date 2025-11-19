
use std::{collections::HashMap};

use evm_rpc_types::Hex20;

use crate::{job::{Job, JobRequest}, state::read_state};

/// Stores all state related to a specific blockchain.
pub struct ChainState {
    // The chain ID using CAIP-2 format.
    pub chain_id: String,

    // The bridge contract address on this chain (e.g., EVM 0x... address).
    pub bridge_address: Address,

    // The highest block number that has been synced.
    pub synced_block_number: Option<u64>,

    // A mapping of job IDs to their corresponding Job structs.
    // TODO: In case of block re-orgs we might see multiple jobs with the same job ID.
    // So maybe better to switch to a hash? 
    pub jobs: HashMap<u64, Job>,

    // A queue of job IDs that still need to be processed.
    // TODO: Probably move to one queue for all chains?
    pub job_queue: Vec<u64>,

    // TODO: Add balances here.
}

impl ChainState {
    pub fn new(chain_id: String, bridge_address: Address) -> Self {
        Self {
            chain_id,
            bridge_address,
            synced_block_number: None,
            jobs: HashMap::new(),
            job_queue: Vec::new(),
        }
    }
}


/// A generic address type that can represent addresses from different blockchain types.
#[derive(Debug, Clone)]
pub enum Address {
    EvmAddress(Hex20)
} 

/// Fetches new jobs from the given chain.
/// TODO: Change return type to Vec<Job> and latest block.
pub async fn fetch_jobs(chain_id: String) -> Result<Vec<JobRequest>, String> {
    match chain_id.as_str() {
        // TODO: Support all EVM chains here.
        "eip155:31337" => {
            let (bridge_address, synced_block_number) = read_state(|state| {
                state.chains.get(&chain_id)
                    .map(|state| (state.bridge_address.clone(), state.synced_block_number))
                    .ok_or_else(|| format!("Chain not supported: {}", chain_id))
            })?;
            let bridge_address = match &bridge_address {
                Address::EvmAddress(addr) => addr.to_string(),
            };
            let evm_chain_id = 31337;  // TODO: Parse from chain_id
            let since_block = synced_block_number.map(|block| block + 1).unwrap_or(0);

            let jobs = crate::evm::fetch_jobs(evm_chain_id, bridge_address, since_block).await?;
            Ok(jobs)
        }
        _ => Err(format!("Unsupported chain id: {}", chain_id)),
    }
}
