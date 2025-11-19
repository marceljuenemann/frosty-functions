mod evm;
mod job;
mod chain;
mod state;

use evm_rpc_types::Hex20;
use wasmi::*;

use chain::{ChainState, Address};
use state::{mutate_state};

use crate::job::Job;

// Host functions that will be available to AssemblyScript
fn ic_time_host() -> i64 {
    ic_cdk::api::time() as i64
}

fn ic_random_host() -> i32 {
    // Simple pseudo-random number for demo
    (ic_cdk::api::time() % 1000) as i32
}

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

fn exec_wasm() -> Result<(i64, u64), String> {
    exec_wasm_with_limit(1_000_000)
}

fn exec_wasm_with_limit(max_instructions: u64) -> Result<(i64, u64), String> {
    ic_cdk::println!("Loading WASM module...");
    let wasm = include_bytes!("../../assembly-playground/build/debug.wasm");
    
    // Create engine with fuel consumption enabled
    let mut config = wasmi::Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    
    let module = Module::new(&engine, &wasm[..]).unwrap();
    ic_cdk::println!("WASM module loaded, setting up linker...");
    let mut linker = <wasmi::Linker<()>>::new(module.engine());
    let mut store = wasmi::Store::new(module.engine(), ());
    
    // Set instruction limit (fuel)
    store.set_fuel(max_instructions).map_err(|e| format!("Failed to set fuel: {}", e))?;
    ic_cdk::println!("Set instruction limit to {} instructions", max_instructions);

    // Register host functions that AssemblyScript can import
    linker
        .define("env", "ic_time_host", Func::wrap(&mut store, ic_time_host))
        .map_err(|e| format!("Failed to define ic_time: {}", e))?;
    linker
        .define("env", "ic_random_host", Func::wrap(&mut store, ic_random_host))
        .map_err(|e| format!("Failed to define ic_random: {}", e))?;
    
    // Standard AssemblyScript runtime functions
    linker
        .define("env", "abort", Func::wrap(&mut store, abort_host))
        .map_err(|e| format!("Failed to define abort: {}", e))?;
    linker
        .define("env", "console.log", Func::wrap(&mut store, console_log_host))
        .map_err(|e| format!("Failed to define console.log: {}", e))?;
    
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();

    ic_cdk::println!("Executing WASM function...");
    
    let run = instance.get_typed_func::<(), i64>(&store, "run").unwrap();
    
    // Execute with resumable fuel handling
    let mut total_consumed = 0u64;
    let mut batch_count = 0u32;
    
    let result = loop {
        match run.call(&mut store, ()) {
            Ok(result) => {
                // Execution completed successfully
                let remaining_fuel = store.get_fuel().unwrap_or(0);
                let batch_consumed = max_instructions - remaining_fuel;
                total_consumed += batch_consumed;
                
                ic_cdk::println!("Execution completed in {} batches, total instructions: {}", 
                               batch_count + 1, total_consumed);
                break result;
            }
            Err(e) if e.to_string().contains("fuel") => {
                // Out of fuel - execution was suspended
                batch_count += 1;
                total_consumed += max_instructions; // We consumed all fuel in this batch
                
                ic_cdk::println!("Batch {} completed: {} instructions consumed (total: {})", 
                               batch_count, max_instructions, total_consumed);
                
                // Check if we should continue (prevent infinite loops)
                if batch_count >= 10 {
                    return Err(format!("Execution suspended after {} batches ({}+ instructions). Possible infinite loop.", 
                                     batch_count, total_consumed));
                }
                
                // Add more fuel to continue execution
                store.set_fuel(max_instructions).map_err(|e| format!("Failed to refuel: {}", e))?;
                ic_cdk::println!("Resuming execution with {} more instructions...", max_instructions);
                
                // Continue the loop to resume execution
                continue;
            }
            Err(e) => {
                return Err(format!("WASM execution failed: {}", e));
            }
        }
    };
    
    Ok((result, total_consumed))
}

#[ic_cdk::update]
fn run_wasm() -> String {
    ic_cdk::println!("Starting WASM execution...");
    ic_cdk::println!("Instruction counter before execution: {}", ic_cdk::api::instruction_counter());

    let (sum, wasm_instructions) = exec_wasm().unwrap();
    ic_cdk::println!("WASM execution completed with sum: {}", sum);
    ic_cdk::println!("Instruction counter after execution: {}", ic_cdk::api::instruction_counter());

    format!(
        "ic_instructions: {}, wasm_instructions: {}, sum: {}",
        ic_cdk::api::performance_counter(0),
        wasm_instructions,
        sum
    )
}

#[ic_cdk::update]
fn run_wasm_with_limit(max_instructions: u64) -> String {
    ic_cdk::println!("Starting WASM execution with limit: {} instructions", max_instructions);
    
    let (sum, wasm_instructions) = exec_wasm_with_limit(max_instructions).unwrap();
    ic_cdk::println!("WASM execution completed with sum: {}", sum);
    
    format!(
        "ic_instructions: {}, wasm_instructions: {}, sum: {}, limit: {}",
        ic_cdk::api::performance_counter(0),
        wasm_instructions,
        sum,
        max_instructions
    )
}

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::query]
fn get_job_info(chain_id: String, job_id: u64) -> Result<Job, String> {
    state::read_chain_state(&chain_id, |state| {
        state.jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| format!("Job not found: {}", job_id))
    })
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
