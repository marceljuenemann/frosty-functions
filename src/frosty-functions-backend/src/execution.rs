use evm_rpc_types::Nat256;
use wasmi::{Engine, Func, Linker, Module, Store};

use crate::{job::JobRequest, state::read_chain_state};

pub async fn execute_job(chain_id: String, job_id: u64) -> Result<(), String> {
    let request = read_chain_state(&chain_id, |state| {
        state.jobs.get(&job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))
            .map(|job| job.request.clone())
    })?;
    let binary = include_bytes!("../../assembly-playground/build/debug.wasm");
    
    // TODO: Verify status is "pending" and set to "in progress".

    execute_binary(&request, binary).await
}

// TODO: Return an Execution error that can be logged.
async fn execute_binary(request: &JobRequest, wasm: &[u8]) -> Result<(), String> {
    let mut config = wasmi::Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);

    let module = Module::new(&engine, &wasm[..]).map_err(|e| format!("Failed to load WASM module: {}", e))?;
    let mut store = wasmi::Store::new(module.engine(), ());
    // TODO: Set instruction limit (fuel) based on available gas.
    store.set_fuel(1_000_000).map_err(|e| format!("Failed to set fuel: {}", e))?;

    let mut linker = <wasmi::Linker<()>>::new(module.engine());
    register_host_functions(&mut linker, &mut store)?;

    // TODO: Replace ic_cdk::println with custom logging to the job logs.
    ic_cdk::println!("Executing job: {:?}", request.on_chain_id);
    let instance = linker.instantiate(&mut store, &module)
        .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?
        .start(&mut store)
        .map_err(|e| format!("Failed to start WASM module: {}", e))?;

    


    Err("Not implemented".to_string())
}

fn register_host_functions(linker: &mut Linker<()>, store: &mut Store<()>) -> Result<(), String> {
    linker
        .define("env", "example_host_function", Func::wrap(store, example_host_function))
        .map_err(|e| format!("Failed to define example_host_function: {}", e))?;
    Ok(())
}
    
fn example_host_function() -> i64 {
    ic_cdk::println!("example_host_function invoked");
    ic_cdk::api::time() as i64
}


    /*

fn exec_wasm_with_limit(max_instructions: u64) -> Result<(i64, u64), String> {
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

*/

