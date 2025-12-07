
use std::{collections::HashMap};

use candid::{CandidType, Nat};
use evm_rpc_types::{Hex20};
use serde::Deserialize;

use crate::{job::{Job, JobRequest}, state::{mutate_chain_state, read_chain_state}};

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq, CandidType, Deserialize)]
pub enum Chain {
    Evm(EvmChain)
}

impl Chain {
    pub fn is_testnet(&self) -> bool {
        match self {
            Chain::Evm(evm_chain) => evm_chain.is_testnet(),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq, CandidType, Deserialize)]
pub enum EvmChain {
    ArbitrumOne,
    ArbitrumSepolia,
    Localhost
}

impl EvmChain {
    pub fn is_testnet(&self) -> bool {
        match self {
            EvmChain::ArbitrumSepolia => true,
            EvmChain::ArbitrumOne => false,
            EvmChain::Localhost => true,
        }
    }
}

/// Stores all state related to a specific blockchain.
pub struct ChainState {
    // The bridge contract address on this chain (e.g., EVM 0x... address).
    pub bridge_address: Address,

    // The highest block number that has been synced.
    pub synced_block_number: Option<u64>,

    // A mapping of job IDs to their corresponding Job structs.
    // TODO: In case of block re-orgs we might see multiple jobs with the same job ID.
    // So maybe better to switch to a hash? 
    pub jobs: HashMap<Nat, Job>,

    // A queue of job IDs that still need to be processed.
    // TODO: Probably move to one queue for all chains?
    pub job_queue: Vec<Nat>,

    // TODO: Add balances here.
}

impl ChainState {
    pub fn new(bridge_address: Address) -> Self {
        Self {
            bridge_address,
            synced_block_number: None,
            jobs: HashMap::new(),
            job_queue: Vec::new(),
        }
    }
}


/// A generic address type that can represent addresses from different blockchain types.
#[derive(Debug, Clone, candid::CandidType, serde::Serialize, serde::Deserialize)]
pub enum Address {
    EvmAddress(Hex20)
} 

/// Syncs the given chain up to the latest block.
/// Returns true if new jobs were created.
pub async fn sync_chain(chain: &Chain) -> Result<bool, String> {
    // TODO: Return and update latest block number.
    let new_jobs = fetch_jobs(&chain).await?;
    let has_jobs = !new_jobs.is_empty();
    mutate_chain_state(&chain, |state| {
        for job_request in new_jobs {
            // TODO: on_chain_id is not the best key as it could be duplicate due to re-orgs.
            // It might also be absent for non-EVM chains.
            let job_id: Nat = Nat::from(job_request.on_chain_id.clone().unwrap());
            if state.jobs.contains_key(&job_id) {
                ic_cdk::println!("ERROR: Job already exists: {:?} {:?}", chain, &job_id);
            } else {
                state.jobs.insert(job_id.clone(), Job { request: job_request });
                state.job_queue.push(job_id);
            }
        }
        Ok(has_jobs)
    })
}

/// Fetches new jobs from the given chain.
async fn fetch_jobs(chain: &Chain) -> Result<Vec<JobRequest>, String> {
    match chain {
        Chain::Evm(evm_chain) => {
            let (bridge_address, synced_block_number) = read_chain_state(&chain, |state| {
                match &state.bridge_address {
                    Address::EvmAddress(addr) => {
                        Ok((addr.to_string(), state.synced_block_number))
                    }
                }
            })?;
            // TODO: Write proper sync logic.
            // let since_block = synced_block_number.map(|block| block + 1).unwrap_or(0);
            let since_block = match evm_chain {
                EvmChain::ArbitrumOne => 403018054 - 10,
                EvmChain::ArbitrumSepolia => 217857590 - 1,
                EvmChain::Localhost => 0,
            };
            let jobs = crate::evm::fetch_jobs(evm_chain, bridge_address, since_block).await?;
            Ok(jobs)
        }
        _ => Err(format!("Unsupported chain: {:?}", chain)),
    }
}
