use std::future::Future;
use std::pin::Pin;

use candid::{CandidType};
use wasmi::WasmParams;
use wasmi::{Engine, Module, TypedFunc, core::TrapCode};

use crate::runtime::api::{register_constants, register_host_functions};
use crate::runtime::{Commit, JobRequest, LogEntry, LogType, RuntimeEnvironment};

const FUEL_PER_BATCH: u64 = 1_000_000;

/// Return type for async host functions.
/// 
/// Guests will have to interpret the byte array depending on the function
/// that created the SharedPromise.
/// TODO: Consider replacing String with (ErrorCode, String)?
pub type AsyncResult = Result<Vec<u8>, String>;

pub struct AsyncTask {
    // ID created by the guest to identify the task.
    pub id: i32,
    // Description of the task for logging purposes.
    pub description: String,
    // Future that will produce the result.
    pub future: Pin<Box<dyn Future<Output = AsyncResult>>>,
}

/// Runtime state for a job execution. All methods are synchronous and the caller is expected
/// to handle scheudling of async operations.
pub struct Execution {
    // TODO: Make private, expose anything neceessary via methods.
    pub store: wasmi::Store<ExecutionContext>,
    instance: wasmi::Instance,
    fn_main: TypedFunc<(), ()>,
    fn_resolve: TypedFunc<(i32, i32), ()>,
    fn_reject: TypedFunc<(i32, i32), ()>,
}

impl Execution {
    /// Instantiates the WASM module and runs its main() function.
    pub fn run_main(wasm: &[u8], env: impl RuntimeEnvironment + 'static) -> Result<Self, String> {
        let mut context = ExecutionContext {
            env: Box::new(env),
            commit_context: None,
        };
        context.commit_begin();  // Can't use with_commit here because ownership will move.

        let mut execution = Self::init(wasm, context)?;
        execution.ctx().log(format!("Instantiated WASM module"));
        execution.call(execution.fn_main, ())?;

        execution.ctx().commit_end("main()".to_string());
        // TODO: Return ExecutionResult of main as well.
        Ok(execution)
    }

    /// Instantiates and starts the WASM module.
    fn init(wasm: &[u8], context: ExecutionContext) -> Result<Self, String> {
        // TODO: Cache the engine instead of recreating it for each execution?
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &wasm[..]).map_err(|e| format!("Failed to load WASM module: {}", e))?;
        
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
    pub fn callback(&mut self, task_id: i32, result: &AsyncResult) -> Result<(), String> {
        match result {
            Ok(data) => {
                ic_cdk::println!("Async task #{} completed successfully.", task_id);
                self.ctx().commit_context().shared_buffer = data.clone();
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

    fn ctx(&mut self) -> &mut ExecutionContext {
        self.store.data_mut()
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
    env: Box<dyn RuntimeEnvironment>,
    commit_context: Option<CommitContext>,
}

impl ExecutionContext {
    pub fn env(&self) -> &dyn RuntimeEnvironment {
        &*self.env
    }

    pub fn log(&mut self, message: String) {
        self.commit_context().logs.push(LogEntry { level: LogType::System, message });
    }

    pub fn queue_task(
        &mut self,
        id: i32,
        description: String,
        future: Pin<Box<dyn Future<Output = AsyncResult>>>,
    ) {
        self.log(format!("Queued AsyncTask #{}: {}", id, description));
        self.commit_context().async_tasks.push(AsyncTask {
            id,
            description,
            future,
        });
    }

    /// Executes the given function within a CommitContext. Many operations such as logging
    /// or scheduling async tasks require a CommitContext to be present. After execution,
    /// `commit()` is called on the RuntimeEnvionment to persist the commit.
    fn with_commit<R>(&mut self, title: String, f: impl FnOnce(&mut Self) -> R) -> R {
        self.commit_begin();
        let result = f(self);
        self.commit_end(title);
        result
    }

    fn commit_begin(&mut self) {
        if self.commit_context.is_some() {
            panic!("CommitContext already present");
        }
        self.commit_context = Some(CommitContext::default());
    }

    fn commit_end(&mut self, title: String) {
        let commit = Commit {
            timestamp: ic_cdk::api::time(),
            title: title,
            logs: self.commit_context().logs.clone(),
        };
        self.env.commit(commit);
        self.commit_context = None;
    }

    pub fn commit_context(&mut self) -> &mut CommitContext {
        self.commit_context.as_mut().expect("CommitContext missing")
    }
}

/// Context valid for a single commit of the exeuction.
#[derive(Default)]
pub struct CommitContext {
    // Logs written during the current commit.
    pub logs: Vec<LogEntry>,
    // Async tasks to schedule.
    pub async_tasks: Vec<AsyncTask>,
    // Shared buffer that the guest can read using copy_shared_buffer.
    pub shared_buffer: Vec<u8>,
}
