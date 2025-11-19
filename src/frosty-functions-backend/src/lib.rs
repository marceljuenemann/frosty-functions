mod chain;
mod evm;
mod execution;
mod job;
mod state;

use chain::{ChainState, Address};
use job::Job;
use state::{mutate_state};

use evm_rpc_types::Hex20;

/*
// Standard AssemblyScript host functions
fn abort_host(message_ptr: i32, file_ptr: i32, line: i32, column: i32) {
    ic_cdk::println!("AssemblyScript abort called at line {} column {} (message_ptr: {}, file_ptr: {})", 
                     line, column, message_ptr, file_ptr);
    ic_cdk::trap("AssemblyScript abort");
}

fn console_log_host(mut caller: wasmi::Caller<'_, ()>, message_ptr: i32) {
    let ptr = message_ptr as u32 as usize;
    if ptr < 4 {
        ic_cdk::println!("AssemblyScript console.log called with invalid pointer: {}", message_ptr);
        return;
    }

    // Get the guest memory
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => {
            ic_cdk::println!("AssemblyScript console.log: no memory export found");
            return;
        }
    };

    // Read byte length stored at (ptr - 4)
    let mut len_buf = [0u8; 4];
    if let Err(e) = memory.read(&mut caller, ptr - 4, &mut len_buf) {
        ic_cdk::println!("console.log: failed reading length: {}", e);
        return;
    }
    let byte_len = u32::from_le_bytes(len_buf) as usize;

    // Read the UTF-16LE bytes
    // TODO: Definie a max length to prevent abuse
    let mut bytes = vec![0u8; byte_len];
    if let Err(e) = memory.read(&mut caller, ptr, &mut bytes) {
        ic_cdk::println!("console.log: failed reading string bytes: {}", e);
        return;
    }

    // Decode UTF-16LE -> Rust String
    let mut u16s = Vec::with_capacity(byte_len / 2);
    for chunk in bytes.chunks_exact(2) {
        u16s.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    match String::from_utf16(&u16s) {
        Ok(s) => ic_cdk::println!("AssemblyScript console.log: {}", s),
        Err(e) => ic_cdk::println!("console.log: invalid UTF-16: {}", e),
    }
}
*/

#[ic_cdk::query]
fn get_job_info(chain_id: String, job_id: u64) -> Result<Job, String> {
    state::read_chain_state(&chain_id, |state| {
        state.jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| format!("Job not found: {}", job_id))
    })
}

#[ic_cdk::query]
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
        "eip155:31337" => {
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
