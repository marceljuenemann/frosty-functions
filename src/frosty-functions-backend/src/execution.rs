use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use alloy::signers::icp::IcpSigner;
use candid::{CandidType};
use ic_cdk_timers::set_timer;
use wasmi::WasmParams;
use wasmi::{Engine, Module, TypedFunc, core::TrapCode};

use crate::api::{register_constants, register_host_functions};
use crate::job::{Commit, JobRequest, JobStatus, LogEntry, LogType};
use crate::signer::signer_for_address;
use crate::storage::{get_function, update_job, update_job_status};

const FUEL_PER_BATCH: u64 = 1_000_000;

pub fn schedule_job(job_request: &JobRequest) {
    let function = get_function(job_request.function_hash.to_vec());
    if function.is_none() {
        update_job_status(&job_request, JobStatus::Failed("No WASM binary found for function".to_string()));
        return;
    }
    
    // Schedule execution of the job in a new IC message in case it panics.
    // TODO: Don't schedule more than X jobs at once.
    let job_request = job_request.clone();
    let timer_id = set_timer(Duration::from_secs(0), async move {
        let wasm = function.unwrap().definition.binary;
        let result = execute_job(job_request, &wasm).await;
        // TODO: Handle errors properly here. Ideally change to void return type.
        if result.is_err() {
            ic_cdk::println!("Job execution failed: {}", result.as_ref().unwrap_err());
        }
    });
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

// TODO: Remove async
pub async fn execute_job(request: JobRequest, wasm: &[u8]) -> Result<(), String> {
    update_job_status(&request, JobStatus::Executing);
    let signer = signer_for_address(&request.caller).await?;
    // TODO: Maybe pass int ExecutionContext
    let mut execution = JobExecution::init(request.clone(), wasm, false, signer)?;
    // TODO: Commit after errors.
    execution.call_by_name("main".to_string())?;

    // TODO: Start all async tasks before commiting?
    // TODO: with_commit rather than manual commit calls.
    // TODO: Probably need a on_commit callback for async functions with multiple commits.
    let mut commits = vec![execution.commit("main()".to_ascii_lowercase())?];

    while !execution.store.data().async_tasks.is_empty() {
        ic_cdk::println!("Processing {} async tasks...", execution.store.data().async_tasks.len());

        // TODO: Wait for multiple tasks in parallel using spawn.
        let task = execution.store.data_mut().async_tasks.remove(0);
        let result = task.future.await;
        execution.callback(task.id, &result)?;
        // TODO: Start more tasks.
        // TODO: Set source
        commits.push(execution.commit(format!("Task #{}: {}", task.id, task.description))?);
    }

    // TODO: Handled errors
    update_job_status(&request, JobStatus::Completed);
    Ok(())
}

pub async fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<ExecutionResult, String> {
    Err("Not implemented yet".to_string())
}

/// Runtime state for a job execution. All methods are synchronous and the caller is expected
/// to handle scheudling of async operations.
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
        // TODO: Cache the engine instead of recreating it for each execution?
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
        store.data_mut().log(LogType::System, format!("Instantiating WASM module"));
        let instance = linker.instantiate_and_start(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?;
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
    fn commit(&mut self, title: String) -> Result<Commit, String> {
        let logs = self.store.data().logs.clone();
        self.store.data_mut().logs.clear();  // TODO: Store in stable memory
        Ok(Commit {
            timestamp: ic_cdk::api::time(),
            title,
            logs,
        })
    }
}

// TODO: This might become a SimulationResult?
#[derive(Clone, Debug, CandidType)]
pub struct ExecutionResult {
    pub commits: Vec<Commit>,
    // Add other fields as needed (gas used, state changes, etc.)
}

/// Runtime context available to host functions during execution.
pub struct ExecutionContext {
    pub request: JobRequest,
    // The shared wallet of the caller of the execution.
    pub caller_wallet: IcpSigner,
    pub simulation: bool,
    // Logs written during the current execution. Will be commited
    // to stable memory before yielding execution.
    // TODO: Move into CommitContext
    pub logs: Vec<LogEntry>,
    // Pending async tasks.
    // TODO: Move into CommitContext
    pub async_tasks: Vec<AsyncTask>,
    // Shared buffer that the guest can read using copy_shared_buffer.
    // TODO: Move into CommitContext
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
