use std::collections::VecDeque;

use wasmi::{Caller, Engine, Func, Linker, Module, Store, TypedFunc, WasmParams, WasmResults, core::TrapCode};

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

    let mut execution = JobExecution::init(request.clone(), binary)?;
    let task_main = execution.call_by_name("main".to_string());
    let task_async = example_async();

    ic_cdk::println!("Waiting for task_main");
    log_current_state();
    task_main.await?;
    ic_cdk::println!("Done waiting for task_main");
    log_current_state();

    /*
    ic_cdk::println!("Waiting for task_async");
    log_current_state();
    task_async.await?;
    ic_cdk::println!("Done waiting for task_async");
    log_current_state();
    */

    Ok(())
}

fn log_current_state() {
    ic_cdk::println!("Instruction counter: {:?}", ic_cdk::api::instruction_counter());
    ic_cdk::println!("Call Context Instruction counter: {:?}", ic_cdk::api::call_context_instruction_counter());
}

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
pub struct JobExecution {
    pub store: wasmi::Store<ExecutionContext>,
    pub instance: wasmi::Instance,
    // TODO: Keep a queue of callbacks to invoke and futures to poll.
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

        let mut linker = <wasmi::Linker<ExecutionContext>>::new(module.engine());
        register_host_functions(&mut linker, &mut store)?;

        // TODO: Replace ic_cdk::println with custom logging to the job logs.
        ic_cdk::println!("Executing job: {:?}", request.on_chain_id);
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?
            // TODO: Allow fail if async host functions are called during initialization?
            .start(&mut store)
            // TODO: Might want to handle out of fuel errors differently here.
            .map_err(|e| format!("Failed to start WASM module: {}", e))?;
    
        Ok(Self {
            store,
            instance,
        })
    }

    /// Calls a function of the WASM module by name.
    async fn call_by_name(&mut self, function_name: String) -> Result<(), String> {
        ic_cdk::println!("Executing WASM function {:?}...", function_name);

        // We interrupt and resume execution based on fuel consumption.
        let func = self.instance.get_typed_func::<(), ()>(&self.store, &function_name)
            .map_err(|e| format!("Failed to get function with name: {:?}", function_name))?;
        self.call_func(func).await
    }

    /// Calls a function of the WASM module, handling fuel consumption and errors.
    async fn call_func(&mut self, function: TypedFunc<(), ()>) -> Result<(), String> {
        loop {
            match function.call(&mut self.store, ()) {
                Ok(()) => {
                    // Execution completed successfully
                    let remaining_fuel = self.store.get_fuel().unwrap_or(0);
                    let fuel_consumed = FUEL_PER_BATCH - remaining_fuel;
                    // TODO: Move this away.
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
}

fn register_host_functions(linker: &mut Linker<ExecutionContext>, store: &mut Store<ExecutionContext>) -> Result<(), String> {
    linker
        .define("env", "abort", Func::wrap(&mut *store, abort_host))
        .map_err(|e| format!("Failed to define abort: {}", e))?;
    linker
        .define("env", "example_host_function", Func::wrap(&mut *store, example_host_function))
        .map_err(|e| format!("Failed to define example_host_function: {}", e))?;
    linker
        .define("env", "example_async_host_function", Func::wrap(&mut *store, example_async_host_function))
        .map_err(|e| format!("Failed to define example_async_host_function: {}", e))?;
    Ok(())
}

fn abort_host(message_ptr: i32, file_ptr: i32, line: i32, column: i32) {
    // TODO: Dereference pointers.
    ic_cdk::println!("AssemblyScript abort at {}:{} (msg={}, file={})", 
                     line, column, message_ptr, file_ptr);
    // TODO: Don't trap? Or update job status to "failed"?
    ic_cdk::trap("AssemblyScript abort");
}

fn example_host_function(caller: Caller<ExecutionContext>) -> i64 {
    let context = caller.data();
    ic_cdk::println!("example_host_function invoked for job: {:?}", context.request.on_chain_id);
    ic_cdk::api::time() as i64
}

fn example_async_host_function(mut caller: Caller<ExecutionContext>, callback: i32) {
    ic_cdk::println!("example_async_host_function invoked with callback index: {}", callback);
    caller.data_mut().pending_callbacks.push_back(callback);
    ic_cdk::println!("Pending callbacks: {:?}", caller.data().pending_callbacks.len());
}

async fn example_async() -> Result<i32, String> {
    ic_cdk::println!("example_async invoked");
    crate::chain::sync_chain("eip155:31337".to_string()).await?;
    Ok(42)
}
