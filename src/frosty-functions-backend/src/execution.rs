use std::future::Future;
use std::pin::Pin;

use alloy::signers::icp::IcpSigner;
use candid::{CandidType, Nat};
use evm_rpc_types::Nat256;
use wasmi::WasmParams;
use wasmi::{Engine, Module, TypedFunc, core::TrapCode};

use crate::signer::signer_for_address;
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

/// Return type for async host functions.
/// 
/// Guests will have to interpret the byte array depending on the function
/// that created the SharedPromise.
/// TODO: Consider replacing String with (ErrorCode, String)?
pub type AsyncResult = Result<Vec<u8>, String>;

struct AsyncTask {
    // ID created by the guest to identify the task.
    pub id: i32,
    // Description of the task for logging purposes.
    pub description: String,
    // Future that will produce the result.
    pub future: Pin<Box<dyn Future<Output = AsyncResult>>>,
}

/// Runtime context available to host functions during execution.
pub struct ExecutionContext {
    pub request: JobRequest,
    // The shared wallet of the caller of the execution.
    pub caller_wallet: IcpSigner,
    pub simulation: bool,
    // Logs written during the current execution. Will be commited
    // to stable memory before yielding execution.
    pub logs: Vec<LogEntry>,
    // Pending async tasks.
    pub async_tasks: Vec<AsyncTask>,
    // Shared buffer that the guest can read using copy_shared_buffer.
    pub shared_buffer: Vec<u8>,
}

impl ExecutionContext {
    pub fn log(&mut self, level: LogType, message: String) {
        self.logs.push(LogEntry { level, message });
    }

    pub fn queue_task(
        &mut self,
        id: i32,
        description: String,
        future: Pin<Box<dyn Future<Output = AsyncResult>>>,
    ) {
        self.log(LogType::System, format!("Queued AsyncTask #{}: {}", id, description));
        self.async_tasks.push(AsyncTask {
            id,
            description,
            future,
        });
    }
}

/// Each job execution can be spread across multiple "commits" if
/// async functions are used. These correlate to a single ICP message.
#[derive(Clone, Debug, CandidType)]
pub struct Commit {
    pub timestamp: u64,
    pub async_task: Option<(i32, String)>,  // Unset for initial main() commit.
    pub logs: Vec<LogEntry>,
}

#[derive(Clone, Debug, CandidType)]
pub struct ExecutionResult {
    pub commits: Vec<Commit>,
    // Add other fields as needed (gas used, state changes, etc.)
}

pub async fn execute_job(chain: Chain, job_id: Nat256) -> Result<(), String> {

    Err("execute_job currently deactivated".to_string())

    /*
    let request = read_chain_state(&chain, |state| {
        state.jobs.get(job_id.as_ref())
            .ok_or_else(|| format!("Job not found: {}", job_id))
            .map(|job| job.request.clone())
    })?;
    let binary = [];  // include_bytes!("../../assembly-playground/build/debug.wasm");
    
    // TODO: Verify status is "pending" and set to "in progress".

    let mut execution = JobExecution::init(request.clone(), &binary, false)?;
    execution.call_by_name("main".to_string())?;

    // Process any pending callbacks registered by host functions
    // TODO: Support actual async operations. Commit gas and state as needed.
    /*
    while let Some(callback_index) = execution.store.data_mut().pending_callbacks.pop_front() {
        ic_cdk::println!("Executing callback with index: {}", callback_index);
        execution.call_by_reference(callback_index)?;
    }
    */

    Ok(())
    */
}

// TODO: Remove async
pub async fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<ExecutionResult, String> {
    let signer = signer_for_address(&request.caller).await?;

    let mut execution = JobExecution::init(request.clone(), wasm, true, signer)?;
    // TODO: Commit after errors.
    execution.call_by_name("main".to_string())?;

    // TODO: Start all async tasks before commiting.
    let mut commits = vec![execution.commit(None)?];

    /*
    if execution.store.data_mut().async_tasks.len() > 0 {
        return Err("Async callbacks not supported in simulation yet".to_string());
    }
    */

    while !execution.store.data().async_tasks.is_empty() {
        ic_cdk::println!("Processing {} async tasks...", execution.store.data().async_tasks.len());

        // TODO: Wait for multiple tasks in parallel using spawn.
        let task = execution.store.data_mut().async_tasks.remove(0);
        let result = task.future.await;
        execution.callback(task.id, &result)?;
        // TODO: Start more tasks.
        // TODO: Set source
        commits.push(execution.commit(Some((task.id, task.description)))?);
    }

    Ok(ExecutionResult { commits })
}

/// Runtime state for a job execution.
// TODO: Maybe don't need this, inline into exeuction function again.
struct JobExecution {
    pub store: wasmi::Store<ExecutionContext>,
    pub instance: wasmi::Instance,
    pub fn_main: TypedFunc<(), ()>,
    pub fn_resolve: TypedFunc<(i32, i32), ()>,
    pub fn_reject: TypedFunc<(i32, i32), ()>,
    // TODO: Keep a queue of callbacks to invoke and futures to poll.
}

impl JobExecution {
    /// Creates a wasmi Engine and initializes the module. 
    pub fn init(request: JobRequest, wasm: &[u8], simulation: bool, signer: IcpSigner) -> Result<Self, String> {
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &wasm[..]).map_err(|e| format!("Failed to load WASM module: {}", e))?;
        
        // Create store with execution context
        let context = ExecutionContext {
            request: request.clone(),
            caller_wallet: signer,
            simulation: simulation,
            logs: Vec::new(),
            async_tasks: Vec::new(),
            shared_buffer: Vec::new(),
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
        // TODO: Move to instantiate_and_start
        store.data_mut().log(LogType::System, format!("Instantiating WASM module"));
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?
            // TODO: Allow fail if async host functions are called during initialization?
            .start(&mut store)
            // TODO: Might want to handle out of fuel errors differently here.
            .map_err(|e| format!("Failed to start WASM module: {}", e))?;
            // TODO: Consider using --exportStart with asc to have a fully initialized module.
    
        Ok(Self {
            fn_main: instance.get_typed_func::<(), ()>(&store, "main").map_err(|e| format!("main() function missing: {}", e))?,
            fn_resolve: instance.get_typed_func::<(i32, i32), ()>(&store, "__frosty_resolve").map_err(|e| format!("__frosty_resolve() function missing: {}", e))?,
            fn_reject: instance.get_typed_func::<(i32, i32), ()>(&store, "__frosty_reject").map_err(|e| format!("__frosty_reject() function missing: {}", e))?,
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
        self.call(func, ())
    }

    /// Calls a function of the WASM module, handling fuel consumption and errors.
    fn call<Params: WasmParams>(&mut self, function: TypedFunc<Params, ()>, params: Params) -> Result<(), String> {
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

    /// Executes the callback for the given async task.
    fn callback(&mut self, task_id: i32, result: &AsyncResult) -> Result<(), String> {
        match result {
            Ok(data) => {
                ic_cdk::println!("Async task #{} completed successfully.", task_id);
                self.store.data_mut().shared_buffer = data.clone();
                self.call(self.fn_resolve, (task_id, data.len() as i32))?;
                Ok(())
                // TODO: commit
            }
            Err(e) => {
                // TODO: Handle properly
                ic_cdk::println!("Async task #{} failed.", task_id);
                return Err(format!("Async task #{} failed: {}", task_id, e));
            }
        }
    }

    /// Commits the current execution state, returning a Commit object.
    /// State should be commited before yielding execution for async operations.
    fn commit(&mut self, async_task: Option<(i32, String)>) -> Result<Commit, String> {
        let logs = self.store.data().logs.clone();
        self.store.data_mut().logs.clear();  // TODO: Store in stable memory
        Ok(Commit {
            timestamp: ic_cdk::api::time(),
            async_task,
            logs,
        })
    }
}
