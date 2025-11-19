
use std::collections::HashMap;

use evm_rpc_types::Hex20;

use crate::{evm::fetch_jobs, job::Job};

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

    /// Fetches new jobs from the chain asynchronously without updating the state.
    /// TODO: Change return type to Vec<Job> and latest block.
    pub async fn fetch_jobs(&self) -> Result<bool, String> {
        match self.chain_id.as_str() {
            // TODO: Support all EVM chains here.
            "eip155:31337" => {
                ic_cdk::println!("Syncing chain: {}", self.chain_id);
                let contract_address = match &self.bridge_address {
                    Address::EvmAddress(addr) => addr.to_string(),
                };

                // TODO: Only fetch new logs.
                let jobs = fetch_jobs(31337, contract_address, 0).await?;
                for job in jobs.into_iter() {
                    ic_cdk::println!("Fetched job: {:?}", job);
                }

                Ok(false)
            }
            _ => Err(format!("Unsupported chain id: {}", self.chain_id)),
        }
    }
}


/// A generic address type that can represent addresses from different blockchain types.
#[derive(Debug, Clone)]
pub enum Address {
    EvmAddress(Hex20)
} 
