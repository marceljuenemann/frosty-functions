use crate::state::read_chain_state;




pub async fn execute_job(chain_id: String, job_id: u64) -> Result<(), String> {
    let request = read_chain_state(&chain_id, |state| {
        state.jobs.get(&job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))
            .map(|job| job.request.clone())
    })?;
    
    // TODO: Set status to "in progress".
    // TODO: Replace ic_cdk::println with custom logging to the job logs.
    ic_cdk::println!("Executing job: {:?} {:?}", chain_id, job_id);
    Ok(())
}

/*

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

*/

