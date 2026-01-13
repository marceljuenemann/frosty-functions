use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use wasmi::{Error, Linker, WasmParams};
use wasmi::{Engine, Module, TypedFunc, core::TrapCode};

use crate::runtime::api::{register_constants, register_host_functions};
use crate::runtime::{Commit, LogEntry, LogType, RuntimeEnvironment};

// Maximum number of host (IC) instructions per job.
// TODO: Increase this except for simulations.
// TODO: Remove the limit altogether by yielding control back to IC.
const HOST_INSTRUCTION_LIMIT: u64 = 1_000_000_000;

// Number of (guest) WASM instructions before we check gas/fuel again.
// TODO: Calling back into a host function can be very expensive, so we
// may still hit the host limit before checking. Ideally we would check within
// each host function call as well. Possibly we can set the remaining fuel to
// zero from there.
const FUEL_PER_BATCH: u64 = 10_000_000;

// Conversion rate between cycles and native currency (wei).
// TODO: Calculate dynamically based on XDR:ETH price.
// Note: Add the time of writing, 1 cycle costs approximatey 430 wei,
// but we need to leave some margin for price fluctuations.
const WEI_PER_CYCLE: u64 = 1000;

/// Runtime state for a job execution. All methods are synchronous and the caller is expected
/// to handle scheudling of async operations.
pub struct Execution {
    store: wasmi::Store<Rc<RefCell<ExecutionContext>>>,
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
            queued_futures: VecDeque::new(),
        };
        context.commit_begin();  // Can't use with_commit here because ownership will move.

        let context = Rc::new(RefCell::new(context));
        let mut execution = Self::init(wasm, context)?;
        execution.ctx().borrow_mut().log(format!("WASM module instantiated"));
        execution.call(execution.fn_main, ())?;
        execution.ctx().borrow_mut().commit_end("main()".to_string());
        // TODO: Return ExecutionResult of main as well.
        Ok(execution)
    }

    /// Instantiates and starts the WASM module.
    fn init(wasm: &[u8], context: Rc<RefCell<ExecutionContext>>) -> Result<Self, String> {
        // TODO: Cache the engine instead of recreating it for each execution?
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &wasm[..]).map_err(|e| format!("Failed to load WASM module: {}", e))?;
        
        let mut store = wasmi::Store::new(module.engine(), context);
        // TODO: Set instruction limit (fuel) based on available gas.
        store.set_fuel(FUEL_PER_BATCH).map_err(|e| format!("Failed to set fuel: {}", e))?;

        // Create linker with host functions and constants.
        let mut linker = Linker::new(module.engine());
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
                        return Err(format!("Instruction limit reached: The number of instructions is currently limited to {}", FUEL_PER_BATCH));
                    }
                    // Other execution error (trap, validation, etc.)
                    return Err(format!("WASM execution failed: {}", e));
                }
            }
        }
    }

    /// Executes the callback for the given AsyncResult.
    pub fn callback(&mut self, result: AsyncResult) -> Result<(), String> {
        match result.result {
            Ok(data) => {
                let title = format!("Resolving Promise #{}: {}", result.promise_id, result.description);
                self.with_commit(title, |exec| {
                    exec.ctx().borrow_mut().commit_context().shared_buffer = data.clone();
                    exec.call(exec.fn_resolve, (result.promise_id, data.len() as i32))?;
                    Ok(())
                })
            }
            Err(err) => {
                let title = format!("Rejecting Promise #{}: {}", result.promise_id, result.description);
                self.with_commit(title, |exec| {
                    // TODO: Proper error serialization with an error code.
                    exec.ctx().borrow_mut().log(format!("Promise rejected with error: {}", err));
                    let err_bytes = err.as_bytes().to_vec();  // TODO: Convert to UTF-16?
                    let err_len = err_bytes.len() as i32;
                    exec.ctx().borrow_mut().commit_context().shared_buffer = err_bytes;
                    exec.call(exec.fn_reject, (result.promise_id, err_len))?;
                    Ok(())
                })
            }
        }
    }

    // TODO: Return all at once
    pub fn next_queued_future(&mut self) -> Option<AsyncFuture> {
        self.ctx().borrow_mut().queued_futures.pop_front()
    }

    // TODO: Return reference instead. Also have ctx_mut() for mutable access.
    fn ctx(&mut self) -> Rc<RefCell<ExecutionContext>> {
        self.store.data().clone()
    }

    /// Executes the given function within a CommitContext. Many operations such as logging
    /// or scheduling async tasks require a CommitContext to be present. After execution,
    /// `commit()` is called on the RuntimeEnvionment to persist the commit.
    fn with_commit<R>(&mut self, title: String, f: impl FnOnce(&mut Self) -> R) -> R {
        self.ctx().borrow_mut().commit_begin();
        let result = f(self);
        self.ctx().borrow_mut().commit_end(title);
        result
    }    
}

/// Runtime context available to host functions during execution.
pub struct ExecutionContext {
    env: Box<dyn RuntimeEnvironment>,
    // Only set in the context of a commit.
    commit_context: Option<CommitContext>,
    // Queue for Futures that should be spawned.
    queued_futures: VecDeque<AsyncFuture>,
}

impl ExecutionContext {
    pub fn env(&self) -> &dyn RuntimeEnvironment {
        &*self.env
    }

    pub fn commit_context(&mut self) -> &mut CommitContext {
        self.commit_context.as_mut().expect("CommitContext missing")
    }

    pub fn log(&mut self, message: String) {
        self.commit_context().logs.push(LogEntry { level: LogType::System, message });
    }

    /// Charges the given fee in the calling currency. Returns an Error if 
    /// insufficient funds are available.
    // TODO: Change Error type to something better.
    pub fn charge_fee(&mut self, fee: u64) -> Result<(), Error> {
        self.env.as_mut().charge_fee(fee)
            .map_err(|e| Error::new(e))?;
        // Tracking of fees on the commit level is purely for informational
        // purposes to make debugging easier for developers.
        self.commit_context().fees += fee;
        Ok(())
    }

    /// Wrapper around charge_fee that converts cycles to native currency.
    pub fn charge_cycles(&mut self, cycles: u64) -> Result<(), Error> {
        self.charge_fee(cycles * WEI_PER_CYCLE)
    }
    
    pub fn queue_task(
        &mut self,
        // TODO: Generate ID instead.
        id: i32,
        description: String,
        future: AsyncFutureInner,
    ) {
        self.log(format!("Spawned Promise #{}: {}", id, description));
        self.queued_futures.push_back(Box::pin(async move {
            let result = AsyncResult {
                promise_id: id,
                description,
                result: future.await,
            };
            // Note we are not in a commit context here.
            // TODO: Remove this or turn it into an optional debug log.
            ic_cdk::println!("Promise #{} completed.", id);
            result
        }));
    }

    fn commit_begin(&mut self) {
        if self.commit_context.is_some() {
            panic!("CommitContext already present");
        }
        self.commit_context = Some(CommitContext {
            initial_instruction_counter: ic_cdk::api::instruction_counter(),
            logs: Vec::new(),
            shared_buffer: Vec::new(),
            fees: 0,
        });
    }

    fn commit_end(&mut self, title: String) {
        let instructions = ic_cdk::api::instruction_counter() - self.commit_context().initial_instruction_counter;
        // TODO: Handle insufficient funds here without failing the entire execution.
        self.charge_cycles(instructions);
        let commit = Commit {
            timestamp: ic_cdk::api::time(),
            title: title,
            logs: self.commit_context().logs.clone(),
            instructions,
            fees: self.commit_context().fees,
        };
        self.env.commit(commit);
        self.commit_context = None;
    }
}

/// Context valid for a single commit of the exeuction.
pub struct CommitContext {
    // Instruction counter at the beginning of the commit.
    pub initial_instruction_counter: u64,
    // Logs written during the current commit.
    pub logs: Vec<LogEntry>,
    // Shared buffer that the guest can read using copy_shared_buffer.
    pub shared_buffer: Vec<u8>,
    // Fees incurred during the commit so far.
    pub fees: u64,
}

pub struct AsyncResult {
    promise_id: i32,
    description: String,
    // TODO: Consider replacing String with (ErrorCode, String)?
    result: AsyncResultInner
}

pub type AsyncFuture = Pin<Box<dyn Future<Output = AsyncResult> + 'static>>;
type AsyncResultInner = Result<Vec<u8>, String>;
type AsyncFutureInner = Pin<Box<dyn Future<Output = AsyncResultInner> + 'static>>;
