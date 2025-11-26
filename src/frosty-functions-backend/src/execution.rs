use std::collections::VecDeque;

use candid::CandidType;
use wasmi::WasmParams;
use wasmi::{Engine, Module, TypedFunc, core::TrapCode};

use crate::{job::JobRequest, state::read_chain_state};
use crate::api::{register_constants, register_host_functions};
use crate::chain::Chain;

const FUEL_PER_BATCH: u64 = 1_000_000;

/**
 * Log entry with different log levels.
 */
#[derive(Clone, Debug, CandidType)]
pub struct LogEntry {
    pub level: LogType,
    pub message: String,
}

#[derive(Clone, Debug, CandidType)]
pub enum LogType {
    System,
    Default,
}

/// Runtime context available to host functions during execution.
pub struct ExecutionContext {
    pub request: JobRequest,
    pub simulation: bool,
    // Logs written during the current execution. Will be commited
    // to stable memory before yielding execution.
    pub logs: Vec<LogEntry>,
    // Queue of function references to invoke in the WASM module.
    pub pending_callbacks: VecDeque<i32>,
    // Add other fields as needed (logs, async results, etc.)
}

impl ExecutionContext {
    pub fn log(&mut self, level: LogType, message: String) {
        self.logs.push(LogEntry { level, message });
    }
}

/// Each job execution can be spread across multiple "commits" if
/// async functions are used. These correlate to a single ICP message.
#[derive(Clone, Debug, CandidType)]
pub struct Commit {
    pub timestamp: u64,
    pub source: CommitSource,
    pub logs: Vec<LogEntry>,
}

#[derive(Clone, Debug, CandidType)]
pub enum CommitSource {
    Main,  // Initial execution of main()
}

#[derive(Clone, Debug, CandidType)]
pub struct ExecutionResult {
    pub commits: Vec<Commit>,
    // Add other fields as needed (gas used, state changes, etc.)
}

pub async fn execute_job(chain: Chain, job_id: u64) -> Result<(), String> {
    let request = read_chain_state(&chain, |state| {
        state.jobs.get(&job_id)
            .ok_or_else(|| format!("Job not found: {}", job_id))
            .map(|job| job.request.clone())
    })?;
    let binary = include_bytes!("../../assembly-playground/build/debug.wasm");
    
    // TODO: Verify status is "pending" and set to "in progress".

    let mut execution = JobExecution::init(request.clone(), binary, false)?;
    execution.call_by_name("main".to_string())?;

    // Process any pending callbacks registered by host functions
    // TODO: Support actual async operations. Commit gas and state as needed.
    while let Some(callback_index) = execution.store.data_mut().pending_callbacks.pop_front() {
        ic_cdk::println!("Executing callback with index: {}", callback_index);
        execution.call_by_reference(callback_index)?;
    }

    Ok(())
}

pub fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<ExecutionResult, String> {
    let mut execution = JobExecution::init(request.clone(), wasm, true)?;
    execution.call_by_name("main".to_string())?;
    let commit = execution.commit(CommitSource::Main)?;

    if execution.store.data_mut().pending_callbacks.len() > 0 {
        return Err("Async callbacks not supported in simulation yet".to_string());
    }

    Ok(ExecutionResult {
        commits: vec![commit],
    })
}

/// Runtime state for a job execution.
// TODO: Maybe don't need this, inline into exeuction function again.
struct JobExecution {
    pub store: wasmi::Store<ExecutionContext>,
    pub instance: wasmi::Instance,
    // TODO: Keep a queue of callbacks to invoke and futures to poll.
}

impl JobExecution {
    /// Creates a wasmi Engine and initializes the module. 
    pub fn init(request: JobRequest, wasm: &[u8], simulation: bool) -> Result<Self, String> {
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &wasm[..]).map_err(|e| format!("Failed to load WASM module: {}", e))?;
        
        // Create store with execution context
        let context = ExecutionContext {
            request: request.clone(),
            simulation: simulation,
            logs: Vec::new(),
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
        store.data_mut().log(LogType::System, format!("Instantiating WASM module"));
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

    /// Calls a function of the WASM module by name.
    fn call_by_name(&mut self, function_name: String) -> Result<(), String> {
        self.store.data_mut().log(LogType::System, format!("Invoking function: {}", function_name));

        // We interrupt and resume execution based on fuel consumption.
        let func = self.instance.get_typed_func::<(), ()>(&self.store, &function_name)
            .map_err(|e| format!("Failed to get function {:?}: {}", function_name, e))?;
        self.call_func(func, ())
    }

    /// Calls a WASM function by table index (function reference).
    /// This is used for callbacks passed from WASM to host functions.
    fn call_by_reference(&mut self, func_index: i32) -> Result<(), String> {
        ic_cdk::println!("Executing WASM callback {:?}...", func_index);

        // We interrupt and resume execution based on fuel consumption.
        // TODO: Allow rejection
        let func = self.instance.get_typed_func::<(i32, i32), ()>(&self.store, "__frosty_resolve")
            .map_err(|e| format!("Failed to get function {:?}: {}", "__frosty_resolve", e))?;
        self.call_func(func, (func_index, 0))
    }

    /// Calls a function of the WASM module, handling fuel consumption and errors.
    fn call_func<Params: WasmParams>(&mut self, function: TypedFunc<Params, ()>, params: Params) -> Result<(), String> {
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

    /// Commits the current execution state, returning a Commit object.
    /// State should be commited before yielding execution for async operations.
    fn commit(&mut self, source: CommitSource) -> Result<Commit, String> {
        let logs = self.store.data().logs.clone();
        self.store.data_mut().logs.clear();  // TODO: Store in stable memory
        Ok(Commit {
            timestamp: ic_cdk::api::time(),
            source,
            logs,
        })
    }
}
