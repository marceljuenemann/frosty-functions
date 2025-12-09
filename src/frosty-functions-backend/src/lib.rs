mod chain;
mod evm;
mod execution;
mod repository;
mod runtime;
mod signer;
mod simulation;
mod state;
mod storage;

use alloy::{signers::Signer};
use chain::{Chain};
use evm_rpc_types::Nat256;

use crate::{execution::schedule_job, repository::{DeployResult, FunctionDefinition, FunctionId, FunctionState}, runtime::{Commit, Job, JobRequest}, simulation::SimulationResult, state::{init_state, read_state}};

#[ic_cdk::update]
async fn init() {
    // TODO: Restrict to controllers.
    init_state().await;
}

#[ic_cdk::query]
fn get_commit(commit_id: u64) -> Option<Commit> {
    crate::storage::get_commit(commit_id)
}

#[ic_cdk::query]
fn get_evm_address() -> String {
    read_state(|state| state.main_signer.address().to_string())
}

/// Retrieve function definition and state by its ID.
#[ic_cdk::query]
fn get_function(id: FunctionId) -> Option<FunctionState> {
    crate::storage::get_function(id)
}

#[ic_cdk::query]
fn get_job(chain: Chain, job_id: Nat256) -> Option<Job> {
    crate::storage::get_job(&chain, job_id.into())
}

/// Deploy a new function.
#[ic_cdk::update]
fn deploy_function(definition: FunctionDefinition) -> DeployResult {
    crate::repository::deploy_function(definition)
}

/// Looks for jobs in the specified block on the given chain.
/// TODO: Currently this call is exposed to the public and invoked from the frontend.
/// This is problematic as the call incurs costs the RPC and could be used to drain cycles.
/// In the future we could require addition of cycles to the call that are refunded only
/// if new jobs were found in the block. We should also provide an (off chain?) indexer to
/// watch for new blocks and call this method automatically.  
#[ic_cdk::update]
async fn index_block(chain: Chain, block_number: u64) -> Result<Vec<JobRequest>, String> {
    match &chain {
        Chain::Evm(evm_chain) => {
            let jobs = crate::evm::index_block(evm_chain, block_number).await?;
            for job in &jobs {
                schedule_job(job);
            }
            Ok(jobs)
        }
    }
}

#[ic_cdk::query]
fn simulate_execution(request: JobRequest, wasm: Vec<u8>) -> Result<SimulationResult, String> {
    crate::simulation::simulate_job(request, &wasm)
}

// Enable Candid export
ic_cdk::export_candid!();
