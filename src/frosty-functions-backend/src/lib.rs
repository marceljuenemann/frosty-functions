use candid::Principal;
use evm_rpc_client::EvmRpcClient;
use evm_rpc_types::{RpcServices, BlockTag, Hex20};
use std::collections::HashMap;
use std::cell::RefCell;
use wasmi::*;
mod job;
mod chain;
use chain::{ChainState};
use crate::job::Job;

thread_local! {
    // Stores the state related to each blockchain we support.
    // NOTE: The canister should be non-upgradeable for security reasons. However,
    // we might still want to move this to stable memory for development purposes.
    static CHAIN_MAP: RefCell<HashMap<String, ChainState>> = RefCell::new(HashMap::new());
}

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

/// Sync jobs from the specified chain.
/// Returns Ok(true) if new jobs were synced.
#[ic_cdk::update]
async fn sync_chain(chain_id: String) -> Result<bool, String> {
    match chain_id.as_str() {
        "eip155:31337" => {
            Ok(false)
        }
        _ => Err(format!("Unsupported chain id: {}", chain_id)),
    }
}

/// Adds a supported chain by its CAIP-2 chain id. Only the owner may call this.
/// Currently supports only specific EVM chains (namespace "eip155").
#[ic_cdk::update]
fn add_chain(chain_id: String, bridge_contract: String) -> Result<bool, String> {
    // TODO: Check that caller is a controller.
    match chain_id.as_str() {
        "eip155:31337" => {
            let inserted = CHAIN_MAP.with(|m| {
                let mut map = m.borrow_mut();
                if map.contains_key(&chain_id) {
                    false
                } else {
                    let chain_state = ChainState::new(chain_id.clone(), bridge_contract.clone());
                    map.insert(chain_id.clone(), chain_state);
                    true
                }
            });
            if inserted { Ok(true) } else { Err("Chain already exists".to_string()) }
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
    // Get the local EVM RPC canister ID from dfx.json (evm_rpc)
    let evm_rpc_canister = Principal::from_text("7hfb6-caaaa-aaaar-qadga-cai")
        .map_err(|e| format!("Invalid canister ID: {}", e))?;

    // Build client with custom localhost URL and local canister ID
    let client = EvmRpcClient::builder(
            evm_rpc_client::IcRuntime,
            evm_rpc_canister,
        )
        .with_rpc_sources(RpcServices::Custom {
            chain_id: 31337, // Local hardhat/anvil chain ID
            services: vec![evm_rpc_types::RpcApi {
                url: "http://127.0.0.1:8545".to_string(),
                headers: None,
            }],
        })
        .build();

    // Convert address string to Hex20
    let address_hex: Hex20 = contract_address
        .parse()
        .map_err(|e| format!("Invalid contract address: {:?}", e))?;

    // Build GetLogsArgs using the From implementation (pass iterator of addresses)
    let mut filter = evm_rpc_types::GetLogsArgs::from(vec![address_hex]);
    filter.from_block = Some(BlockTag::Earliest);
    filter.to_block = Some(BlockTag::Latest);

    // Call getLogs and send the request
    let result = client
        .get_logs(filter)
        .send()
        .await;

    // Format result as JSON-like string
    Ok(format!("{:?}", result))
}
