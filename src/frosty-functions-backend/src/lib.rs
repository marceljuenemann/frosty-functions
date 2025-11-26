mod api;
mod chain;
mod evm;
mod execution;
mod job;
mod state;

use chain::{Chain, EvmChain, ChainState, Address};
use job::Job;
use state::{mutate_state};

use crate::{execution::ExecutionResult, job::JobRequest};

#[ic_cdk::init]
fn init() {
    mutate_state(|state| {
        // TODO: Make bridge addresses configurable.
        let local_bridge_address: Address = Address::EvmAddress("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".parse().unwrap());
        let bridge_address: Address = Address::EvmAddress("0xe712a7e50aba019a6d225584583b09c4265b037b".parse().unwrap());
        state.chains.insert(Chain::Evm(EvmChain::ArbitrumOne), ChainState::new(bridge_address.clone()));
        state.chains.insert(Chain::Evm(EvmChain::ArbitrumSepolia), ChainState::new(bridge_address.clone()));
        state.chains.insert(Chain::Evm(EvmChain::Localhost), ChainState::new(local_bridge_address.clone()));
    });
}

#[ic_cdk::query]
fn get_job_info(chain: Chain, job_id: u64) -> Result<Job, String> {
    state::read_chain_state(&chain, |state| {
        state.jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| format!("Job not found: {}", job_id))
    })
}

#[ic_cdk::query]
fn simulate_execution(request: JobRequest, wasm: Vec<u8>) -> Result<ExecutionResult, String> {
    crate::execution::simulate_job(request, &wasm)
}

#[ic_cdk::update]
async fn execute_job(chain: Chain, job_id: u64) -> Result<(), String> {
    crate::execution::execute_job(chain, job_id).await
}

/// Fetches new jobs from the specified chain.
/// Returns Ok(true) if new jobs were synced.
#[ic_cdk::update]
async fn sync_chain(chain: Chain) -> Result<bool, String> {
    crate::chain::sync_chain(&chain).await
}

/// Returns IDs of jobs currently in the queue for processing.
#[ic_cdk::query]
async fn get_queue(chain: Chain) -> Result<Vec<u64>, String> {
    state::read_chain_state(&chain, |state| {
        Ok(state.job_queue.clone())
    })
}

// Enable Candid export
ic_cdk::export_candid!();
