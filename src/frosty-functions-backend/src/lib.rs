use wasmi::*;

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

fn console_log_host(message_ptr: i32) {
    ic_cdk::println!("AssemblyScript console.log called (message_ptr: {})", message_ptr);
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
