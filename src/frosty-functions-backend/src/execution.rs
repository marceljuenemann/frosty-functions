use std::collections::VecDeque;

use wasmi::WasmParams;
use wasmi::{Engine, Linker, Module, Store, TypedFunc, core::TrapCode};

use crate::{job::JobRequest, state::read_chain_state};
use crate::api::{register_constants, register_host_functions};

const FUEL_PER_BATCH: u64 = 1_000_000;

/// Runtime context available to host functions during execution.
#[derive(Clone)]
pub struct ExecutionContext {
    pub request: JobRequest,
    // Queue of function references to invoke in the WASM module.
    pub pending_callbacks: VecDeque<i32>,
    // Add other fields as needed (logs, async results, etc.)
}

/// Runtime state for a job execution.
// TODO: Maybe don't need this, inline into exeuction function again.
struct JobExecution {
    pub store: wasmi::Store<ExecutionContext>,
    pub instance: wasmi::Instance,
    // TODO: Keep a queue of callbacks to invoke and futures to poll.
}

pub async fn execute_job(chain_id: String, job_id: u64) -> Result<(), String> {
    let request = read_chain_state(&chain_id, |state| {
        state.jobs.get(&job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))
            .map(|job| job.request.clone())
    })?;
    let binary = include_bytes!("../../assembly-playground/build/debug.wasm");
    
    // TODO: Verify status is "pending" and set to "in progress".

    let mut execution = JobExecution::init(request.clone(), binary)?;
    execution.execute().await?;

    /*
    ic_cdk::println!("Waiting for task_async");
    log_current_state();
    task_async.await?;
    ic_cdk::println!("Done waiting for task_async");
    log_current_state();
    */

    Ok(())
}

fn _log_current_state() {
    ic_cdk::println!("Instruction counter: {:?}", ic_cdk::api::instruction_counter());
    ic_cdk::println!("Call Context Instruction counter: {:?}", ic_cdk::api::call_context_instruction_counter());
}

impl JobExecution {
    /// Creates a wasmi Engine and initializes the module. 
    pub fn init(request: JobRequest, wasm: &[u8]) -> Result<Self, String> {
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &wasm[..]).map_err(|e| format!("Failed to load WASM module: {}", e))?;
        
        // Create store with execution context
        let context = ExecutionContext {
            request: request.clone(),
            pending_callbacks: VecDeque::new(),
        };
        let mut store = wasmi::Store::new(module.engine(), context);
        // TODO: Set instruction limit (fuel) based on available gas.
        store.set_fuel(FUEL_PER_BATCH).map_err(|e| format!("Failed to set fuel: {}", e))?;

        // Create linker with host functions and constants.
        let mut linker = <wasmi::Linker<ExecutionContext>>::new(module.engine());
        register_constants(&mut linker, &mut store)
            .map_err(|e| format!("Failed to register constants: {}", e))?;
        register_host_functions(&mut linker, &mut store)
            .map_err(|e| format!("Failed to register host functions: {}", e))?;

        // Initialize and start the module instance.
        // TODO: Replace ic_cdk::println with custom logging to the job logs.
        ic_cdk::println!("Executing job: {:?}", request.on_chain_id);
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?
            // TODO: Allow fail if async host functions are called during initialization?
            .start(&mut store)
            // TODO: Might want to handle out of fuel errors differently here.
            .map_err(|e| format!("Failed to start WASM module: {}", e))?;
            // TODO: Consider using --exportStart with asc to have a fully initialized module.
    
        Ok(Self {
            store,
            instance,
        })
    }

    /// Executes the main() function as well as any asynchronous callbacks.
    async fn execute(&mut self) -> Result<(), String> {
        self.call_by_name("main".to_string()).await?;

        // Process any pending callbacks registered by host functions
        while let Some(callback_index) = self.store.data_mut().pending_callbacks.pop_front() {
            ic_cdk::println!("Executing callback with index: {}", callback_index);
            self.call_by_reference(callback_index).await?;
        }

        Ok(())
    }

    /// Calls a function of the WASM module by name.
    async fn call_by_name(&mut self, function_name: String) -> Result<(), String> {
        ic_cdk::println!("Executing WASM function {:?}...", function_name);

        // We interrupt and resume execution based on fuel consumption.
        let func = self.instance.get_typed_func::<(), ()>(&self.store, &function_name)
            .map_err(|e| format!("Failed to get function {:?}: {}", function_name, e))?;
        self.call_func(func, ()).await
    }

    /// Calls a WASM function by table index (function reference).
    /// This is used for callbacks passed from WASM to host functions.
    async fn call_by_reference(&mut self, func_index: i32) -> Result<(), String> {
        ic_cdk::println!("Executing WASM callback {:?}...", func_index);

        // We interrupt and resume execution based on fuel consumption.
        // TODO: Allow rejection
        let func = self.instance.get_typed_func::<(i32, i32), ()>(&self.store, "__frosty_resolve")
            .map_err(|e| format!("Failed to get function {:?}: {}", "__frosty_resolve", e))?;
        self.call_func(func, (func_index, 0)).await
    }

    /// Calls a function of the WASM module, handling fuel consumption and errors.
    async fn call_func<Params: WasmParams>(&mut self, function: TypedFunc<Params, ()>, params: Params) -> Result<(), String> {
        loop {
            match function.call(&mut self.store, params) {
                Ok(()) => {
                    // Execution completed successfully
                    let remaining_fuel = self.store.get_fuel().unwrap_or(0);
                    let fuel_consumed = FUEL_PER_BATCH - remaining_fuel;
                    // TODO: Move this away.
                    ic_cdk::println!("WASM call completed. Fuel consumed: {:?}", fuel_consumed);
                    return Ok(());
                }
                Err(e) => {
                    if let Some(TrapCode::OutOfFuel) = e.as_trap_code() {
                        // TODO: Deduct cycles from gas, then resume execution if more gas is available.
                        // self.store.set_fuel(FUEL_PER_BATCH).map_err(|e| format!("Failed to refuel: {}", e))?;
                        return Err("WASM execution ran out of fuel".to_string());
                    }
                    // Other execution error (trap, validation, etc.)
                    return Err(format!("WASM execution failed: {}", e));
                }
            }
        }
    }
}

