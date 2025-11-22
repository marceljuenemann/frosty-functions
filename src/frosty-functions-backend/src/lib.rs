mod api;
mod chain;
mod evm;
mod execution;
mod job;
mod state;

use chain::{ChainState, Address};
use job::Job;
use state::{mutate_state};

use evm_rpc_types::Hex20;

#[ic_cdk::query]
fn get_job_info(chain_id: String, job_id: u64) -> Result<Job, String> {
    state::read_chain_state(&chain_id, |state| {
        state.jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| format!("Job not found: {}", job_id))
    })
}

#[ic_cdk::update]
async fn execute_job(chain_id: String, job_id: u64) -> Result<(), String> {
    crate::execution::execute_job(chain_id, job_id).await
}

/// Fetches new jobs from the specified chain.
/// Returns Ok(true) if new jobs were synced.
#[ic_cdk::update]
async fn sync_chain(chain_id: String) -> Result<bool, String> {
    crate::chain::sync_chain(chain_id).await
}

/// Returns IDs of jobs currently in the queue for processing.
#[ic_cdk::query]
async fn get_queue(chain_id: String) -> Result<Vec<u64>, String> {
    state::read_chain_state(&chain_id, |state| {
        Ok(state.job_queue.clone())
    })
}

/// Adds a supported chain by its CAIP-2 chain id. Only the owner may call this.
/// Currently supports only specific EVM chains (namespace "eip155").
#[ic_cdk::update]
fn add_chain(chain_id: String, bridge_contract: String) -> Result<bool, String> {
    // TODO: Check that caller is a controller.
    match chain_id.as_str() {
        "eip155:421614" => {  // TODO: Support more chains
            mutate_state(|state| {
                if state.chains.contains_key(&chain_id) {
                    Err("Chain already exists".to_string())
                } else {
                    let bridge_address: Hex20 = bridge_contract.parse()
                        .map_err(|e| format!("Invalid bridge contract address: {}", e))?;
                    let chain_state = ChainState::new(chain_id.clone(), Address::EvmAddress(bridge_address));
                    state.chains.insert(chain_id.clone(), chain_state);
                    Ok(true)
                }
            })
        }
        _ => Err(format!("Unsupported chain id: {}", chain_id)),
    }
}

// Get logs from a smart contract using the local EVM RPC canister
// Parameters:
// - contract_address: the contract address (e.g., "0x...")
#[ic_cdk::update]
async fn evm_get_logs(
    contract_address: String,
) -> Result<String, String> {
//    tmp_get_logs(contract_address).await
    Err("Not implemented".to_string())
}

// Enable Candid export
ic_cdk::export_candid!();
