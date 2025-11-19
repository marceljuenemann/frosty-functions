use wasmi::{Engine, Func, Linker, Module, Store, core::TrapCode, errors::{ErrorKind, FuelError}};

use crate::{job::JobRequest, state::read_chain_state};

const FUEL_PER_BATCH: u64 = 1_000_000;

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
    store.set_fuel(FUEL_PER_BATCH).map_err(|e| format!("Failed to set fuel: {}", e))?;

    let mut linker = <wasmi::Linker<()>>::new(module.engine());
    register_host_functions(&mut linker, &mut store)?;

    // TODO: Replace ic_cdk::println with custom logging to the job logs.
    ic_cdk::println!("Executing job: {:?}", request.on_chain_id);
    let instance = linker.instantiate(&mut store, &module)
        .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?
        .start(&mut store)
        // TODO: Might want to handle out of fuel errors differently here.
        .map_err(|e| format!("Failed to start WASM module: {}", e))?;

        ic_cdk::println!("Executing WASM function...");
    
    // We interrupt and resume execution based on fuel consumption.
    let entry_point = instance.get_typed_func::<(), ()>(&store, "main")
        .map_err(|e| format!("Failed to find main() function: {}", e))?;
    loop {
        match entry_point.call(&mut store, ()) {
            Ok(()) => {
                // Execution completed successfully
                let remaining_fuel = store.get_fuel().unwrap_or(0);
                let fuel_consumed = FUEL_PER_BATCH - remaining_fuel;
                ic_cdk::println!("Execution completed. Fuel consumed: {:?}", fuel_consumed);
                return Ok(());
            }
            Err(e) => {
                if let Some(TrapCode::OutOfFuel) = e.as_trap_code() {
                    // TODO: Deduct cycles from gas, then resume execution if more gas is available.
                    // store.set_fuel(FUEL_PER_BATCH).map_err(|e| format!("Failed to refuel: {}", e))?;
                    return Err("WASM execution ran out of fuel".to_string());
                }
                // Other execution error (trap, validation, etc.)
                return Err(format!("WASM execution failed: {}", e));
            }
        }
    }
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
