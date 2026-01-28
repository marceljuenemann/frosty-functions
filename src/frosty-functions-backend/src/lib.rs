mod chain;
mod crosschain;
mod evm;
mod execution;
mod repository;
mod runtime;
mod signer;
mod simulation;
mod state;
mod storage;

use std::cell::RefCell;

use candid::CandidType;
use chain::{Chain};
use evm_rpc_types::Nat256;
use serde::{Deserialize, Serialize};

use crate::{chain::Caller, crosschain::eth_address_for_public_key, execution::schedule_job, repository::{DeployResult, FunctionDefinition, FunctionId, FunctionState}, runtime::{Commit, Job, JobRequest}, signer::main_signer, simulation::SimulationResult, state::{init_state, read_state}};
use crate::crosschain::Signer;

// TODO: Remove again
thread_local! {
    static VALID_API_KEYS: RefCell<Option<Vec<String>>> = RefCell::new(None);
}

#[ic_cdk::update]
async fn init() {
    // TODO: Just trigger from an init hook.
    init_state().await;
}

#[ic_cdk::query]
fn get_commit(commit_id: u64) -> Option<Commit> {
    crate::storage::get_commit(commit_id)
}

#[ic_cdk::query]
fn get_evm_address() -> String {
    read_state(|state| alloy::signers::Signer::address(&state.main_signer).to_string())
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
fn deploy_function(definition: FunctionDefinition, api_key: Option<String>) -> DeployResult {
    let valid_keys = VALID_API_KEYS.with_borrow(|keys| keys.clone().unwrap_or_default());
    if !valid_keys.is_empty() {
        if api_key.is_none() {
            return DeployResult::Error("Deployment is currently restricted to private alpha users. Reach out to frosty@web3.services for an API key".to_string());
        } else if !valid_keys.contains(&api_key.unwrap()) {
            return DeployResult::Error("Invalid API key provided.".to_string());
        }
    }
    crate::repository::deploy_function(definition)
}

#[ic_cdk::update]
fn tmp_set_api_keys(admin_key: String, api_keys: Option<Vec<String>>) -> Result<(), String> {
    let keys = VALID_API_KEYS.with_borrow(|keys| keys.clone());
    if keys.is_some() && keys.unwrap().get(0) != Some(&admin_key) {
        return Err("Invalid admin key".to_string());
    }
    VALID_API_KEYS.replace(api_keys);
    Ok(())
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

#[ic_cdk::query]
fn signer_for_caller(caller: Caller, derivation: Option<Vec<u8>>) -> Result<SignerInfo, String> {
    Signer::for_caller(caller, derivation).into()
}

#[ic_cdk::query]
fn signer_for_function(function_id: FunctionId, derivation: Option<Vec<u8>>) -> Result<SignerInfo, String> {
    Signer::for_function(function_id, derivation).into()
}   

#[derive(Clone, CandidType, Deserialize, Serialize)]
struct SignerInfo {
    public_key: String,
    eth_address: String,
}

impl From<Signer> for Result<SignerInfo, String> {
    fn from(signer: Signer) -> Self {
        let public_key = signer.public_key()?;
        let address = eth_address_for_public_key(&public_key).map_err(|e| e.to_string())?;
        Ok(SignerInfo {
            public_key: hex::encode(&public_key),
            eth_address: address.to_string(),
        })
    }
}

ic_cdk::export_candid!();
