use std::{cmp::min};

use wasmi::{Caller, Error, Func, Global, Linker, Memory, Mutability, Store, Val, errors::LinkerError};
use crate::{Chain, execution::{ExecutionContext, LogEntry, LogType}};

const CONSOLE_LOG_MAX_LEN: usize = 10_000;

/// Registers all constants into the given linker.
pub fn register_constants(linker: &mut Linker<ExecutionContext>, store: &mut Store<ExecutionContext>) -> Result<(), LinkerError> {
    let calldata_size = store.data().request.data.len() as i32;
    linker.define("❄️", "CALLDATA_SIZE", Global::new(
        &mut *store,
        Val::I32(calldata_size),
        Mutability::Const
    ))?;
    Ok(())
}

/// Registers all host functions into the given linker.
pub fn register_host_functions(linker: &mut Linker<ExecutionContext>, store: &mut Store<ExecutionContext>) -> Result<(), LinkerError> {
    // TODO: Define a macro to reduce boilerplate.
    linker.define("env", "abort", Func::wrap(&mut *store, abort_host))?;
    linker.define("env", "console.log", Func::wrap(&mut *store, console_log))?;
    linker.define("❄️", "calldata", Func::wrap(&mut *store, calldata))?;
    linker.define("❄️", "evm_chain_id", Func::wrap(&mut *store, evm_chain_id))?;
    linker.define("❄️", "on_chain_id", Func::wrap(&mut *store, on_chain_id))?;
    linker.define("❄️", "example_host_function", Func::wrap(&mut *store, example_host_function))?;
    linker.define("❄️", "example_async_host_function", Func::wrap(&mut *store, example_async_host_function))?;
    Ok(())
}

fn abort_host(message_ptr: i32, file_ptr: i32, line: i32, column: i32) {
    ic_cdk::println!("AssemblyScript abort at {}:{} (msg_ptr={}, file_ptr={})", line, column, message_ptr, file_ptr);
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
    ic_cdk::println!("Pending callbacks queued: {}", caller.data().pending_callbacks.len());
}

// TODO: Support console.error etc.
fn console_log(mut caller: Caller<ExecutionContext>, message_ptr: i32) {
    let message = read_utf16_string(&caller, message_ptr, CONSOLE_LOG_MAX_LEN)
        // TODO: Return error?
        .unwrap_or_else(|e| format!("(failed to read log message: {})", e));
    caller.data_mut().log(LogType::Default, message.clone());
    ic_cdk::println!("console.log: {}", message);  // TODO: remove
    // TODO: Charge cycles for logs storage.
}

fn evm_chain_id(caller: Caller<ExecutionContext>) -> u64 {
    match &caller.data().request.chain  {
        Chain::Evm(id) => crate::evm::evm_chain_id(id.clone()),
        _ => 0,
    }
}

/// Writes the calldata into the provided buffer, which is expected to be of CALLDATA_SIZE.
fn calldata(mut caller: Caller<ExecutionContext>, buffer_ptr: i32) -> Result<(), Error> {
    let calldata = caller.data().request.data.clone();
    ic_cdk::println!("calldata host function called, writing {} bytes to ptr {}", calldata.len(), buffer_ptr);
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &calldata)?;
    Ok(())  // TODO: remove?
}

fn on_chain_id(caller: Caller<ExecutionContext>) -> i64 {
    let context = caller.data();
    if let Some(id) = context.request.on_chain_id.clone() {
        // TODO: Proper error handling.
        let id: u64 = id.try_into().unwrap();
        id as i64
    } else {
        -1
    }
}

// Reads a UTF-16LE encoded string from the guest memory at the given pointer.
// TODO: What about error handling? Host function should be able to return Result as well?
fn read_utf16_string(caller: &wasmi::Caller<ExecutionContext>, str_ptr: i32, max_len: usize) -> Result<String, Error> {
    let bytes = read_buffer(caller, str_ptr, max_len * 2)?;
    let mut u16s = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        u16s.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16(&u16s)
        .map_err(|e| Error::new(format!("Invalid UTF-16 string: {}", e)))
}

/// Reads a buffer from the guest memory at the given pointer.
/// The length of the buffer is presumed to be in the first 4 bytes before the pointer.
fn read_buffer(caller: &Caller<ExecutionContext>, ptr: i32, max_len: usize) -> Result<Vec<u8>, Error> {
    let memory = get_memory(caller);

    // Read buffer length stored at (ptr - 4)
    let ptr = ptr as u32 as usize;
    let mut buf_len = [0u8; 4];
    memory.read(caller, ptr - 4, &mut buf_len)
        .map_err(|e| Error::new(format!("Failed reading buffer length: {}", e)))?;
    let buf_len = min(u32::from_le_bytes(buf_len) as usize, max_len);

    // Read the bytes into the buffer
    let mut bytes = vec![0u8; buf_len];
    memory.read(caller, ptr, &mut bytes)
        .map_err(|e| Error::new(format!("Failed reading buffer: {}", e)))?;
    Ok(bytes)
}

fn get_memory(caller: &Caller<ExecutionContext>) -> Memory {
    caller
        .get_export("memory")
        .and_then(|ext| ext.into_memory())
        .expect("Invalid WASM module: No memory found")
}
