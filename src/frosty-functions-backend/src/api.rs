use std::{cmp::min};

use alloy::primitives::Address;
use alloy::signers::Signer;
use wasmi::{Caller, Error, Func, Global, Linker, Memory, Mutability, Store, Val, errors::LinkerError};
use crate::{Chain, chain::EvmChain, evm::transfer_funds, execution::{ExecutionContext, LogType}};

const CONSOLE_LOG_MAX_LEN: usize = 10_000;

macro_rules! log {
    ($caller:expr, $($arg:tt)*) => {
        $caller.data_mut().log(LogType::System, format!($($arg)*));
    };
}

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

macro_rules! register {
    ($func:expr, $linker:expr, $store:expr) => {
        $linker.define("❄️", stringify!($func), Func::wrap(&mut *$store, $func))?;
    };
}

/// Registers all host functions into the given linker.
pub fn register_host_functions(linker: &mut Linker<ExecutionContext>, store: &mut Store<ExecutionContext>) -> Result<(), LinkerError> {
    linker.define("env", "abort", Func::wrap(&mut *store, abort_host))?;
    linker.define("env", "console.log", Func::wrap(&mut *store, console_log))?;
    linker.define("env", "seed", Func::wrap(&mut *store, seed))?;

    register!(calldata, linker, store);
    register!(copy_shared_buffer, linker, store);
    register!(on_chain_id, linker, store);

    register!(evm_callback, linker, store);
    register!(evm_caller_wallet_address, linker, store);
    register!(evm_chain_id, linker, store);

    register!(ic_raw_rand, linker, store);

    Ok(())
}

fn abort_host(message_ptr: i32, file_ptr: i32, line: i32, column: i32) {
    ic_cdk::println!("AssemblyScript abort at {}:{} (msg_ptr={}, file_ptr={})", line, column, message_ptr, file_ptr);
    ic_cdk::trap("AssemblyScript abort");
}

fn seed() -> Result<f64, Error> {
    // TODO: Require asynchronous initialization first. In fact, will need
    // to provide a separate API as we can't make seed() asynchronous.
    // caller.data().log(LogType::System, "Seeding randomness with VRF");
    // return ic_cdk::api::management_canister::main::raw_rand() as i64;
    Err(Error::new("Verifiable Random Function not yet implemented"))
}

// TODO: Support console.error etc.
fn console_log(mut caller: Caller<ExecutionContext>, message_ptr: i32) {
    let message = read_utf16_string(&caller, message_ptr, CONSOLE_LOG_MAX_LEN)
        // TODO: Return error?
        .unwrap_or_else(|e| format!("(failed to read log message: {})", e));
    caller.data_mut().log(LogType::Default, message.clone());
    // TODO: Charge cycles for logs storage.
}

/// Writes the calldata into the provided buffer, which is expected to be of CALLDATA_SIZE.
fn calldata(mut caller: Caller<ExecutionContext>, buffer_ptr: i32) -> Result<(), Error> {
    let calldata = caller.data().request.data.clone();
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &calldata)?;
    Ok(())  // TODO: remove?
}

fn on_chain_id(caller: Caller<ExecutionContext>) -> i64 {
    let context = caller.data();
    if let Some(id) = context.request.on_chain_id.clone() {
        // TODO: Proper error handling for overflows.
        let id: u64 = id.to_string().parse().unwrap();
        id as i64
    } else {
        -1
    }
}

/// Writes the caller's wallet address as a UTF-16LE string into the provided buffer.
/// The buffer is expected to be large enough to hold the address string.
fn evm_caller_wallet_address(mut caller: Caller<ExecutionContext>, buffer_ptr: i32) -> Result<(), Error> {
    let address = caller.data().signer.address().to_string();
    ic_cdk::println!("EVM caller wallet address: {}", address);
    write_utf16_string(&mut caller, &address, buffer_ptr)
}

fn evm_chain_id(caller: Caller<ExecutionContext>) -> u64 {
    match &caller.data().request.chain  {
        Chain::Evm(id) => crate::evm::evm_chain_id(id.clone()),
        _ => 0,
    }
}

fn evm_callback(mut caller: Caller<ExecutionContext>, promise_id: i32, data_ptr: i32, amount: u64) -> Result<(), Error> {
    // TODO: Add actual arguments
    let recipient: Address = "0xe712a7e50aba019a6d225584583b09c4265b037b".parse().unwrap();
    let recipient: [u8; 20] = recipient.into();
    let data: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF];

    caller.data_mut().queue_task(
        promise_id,
        format!("EVM callback with amount {} and data 0x{}", amount, hex::encode(&data)),
        Box::pin(async move {
            ic_cdk::println!("Hello EVM!");
            transfer_funds(EvmChain::Localhost, "0xe712a7e50aba019a6d225584583b09c4265b037b".to_string(), amount).await?;
            Ok(vec![0u8])
        }) 
    );
    Ok(())
}
    
fn ic_raw_rand(mut caller: Caller<ExecutionContext>, promise_id: i32) -> Result<(), Error> {
    caller.data_mut().queue_task(
        promise_id,
        "Retrieve verifiable randomness".to_string(),
        Box::pin(async {
            Err("Not yet implemented".to_string())
            /*
            ic_cdk::management_canister::raw_rand().await
                .map_err(|e| format!("Failed to get raw_rand: {}", e))
                */
        }) 
    );
    Ok(())
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

/// Writes the given string as UTF-16LE into the guest memory at the given pointer.
fn write_utf16_string(caller: &mut Caller<ExecutionContext>, str: &String, buffer_ptr: i32) -> Result<(), Error> {
    let bytes: Vec<u8> = str.encode_utf16().flat_map(|unit| unit.to_le_bytes()).collect();
    ic_cdk::println!("Writing UTF-16. length: {}", bytes.len());
    get_memory(&caller).write(caller, buffer_ptr as usize, &bytes)?;
    Ok(())
}

/// Copies the shared buffer into the guest memory at the given pointer.
/// The guest is expected to have allocated a buffer of the same size.
fn copy_shared_buffer(mut caller: Caller<ExecutionContext>, buffer_ptr: i32) -> Result<(), Error> {
    let shared_buffer = caller.data().shared_buffer.clone();
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &shared_buffer)?;
    Ok(())
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
