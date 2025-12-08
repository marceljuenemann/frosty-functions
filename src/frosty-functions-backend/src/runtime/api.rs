use alloy::signers::Signer;
use wasmi::{Caller, Error, Func, Global, Linker, Memory, Mutability, Store, Val, errors::LinkerError};
use crate::runtime::{LogEntry, LogType};
use crate::{Chain, evm::transfer_funds};
use crate::runtime::runtime::{ExecutionContext};

/// The maximum length of data that can be passed to/from the guest.
// TODO: Consider increasing if there is a use case.
const BUFFER_MAX_LEN: usize = 10_000_000;

/// The maximum length of console log messages.
const CONSOLE_LOG_MAX_LEN: usize = 10_000;

macro_rules! log {
    ($caller:expr, $($arg:tt)*) => {
        $caller.data_mut().log(LogType::System, format!($($arg)*));
    };
}

/// Registers all constants into the given linker.
pub fn register_constants(linker: &mut Linker<ExecutionContext>, store: &mut Store<ExecutionContext>) -> Result<(), LinkerError> {
    let calldata_size = store.data().env().job_request().data.len() as i32;
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

    register!(evm_caller_wallet_address, linker, store);
    register!(evm_caller_wallet_deposit, linker, store);
    register!(evm_caller_wallet_sign_message, linker, store);
    register!(evm_chain_id, linker, store);

    register!(ic_raw_rand, linker, store);

    Ok(())
}

fn abort_host(message_ptr: i32, file_ptr: i32, line: i32, column: i32) {
    ic_cdk::println!("AssemblyScript abort at {}:{} (msg_ptr={}, file_ptr={})", line, column, message_ptr, file_ptr);
    ic_cdk::trap("AssemblyScript abort");
}

fn seed() -> Result<f64, Error> {
    Err(Error::new("Use the frosty/rand module to retrieve verifiable randomness"))
}

// TODO: Support console.error etc.
fn console_log(mut caller: Caller<ExecutionContext>, message_ptr: i32) {
    let message = read_utf16_string(&caller, message_ptr, CONSOLE_LOG_MAX_LEN)
        // TODO: Return error?
        .unwrap_or_else(|e| format!("(failed to read log message: {})", e));
    caller.data_mut().commit_context().logs.push(LogEntry { level: LogType::Default, message: message.clone() });
    // TODO: Charge cycles for logs storage.
}

/// Writes the calldata into the provided buffer, which is expected to be of CALLDATA_SIZE.
fn calldata(mut caller: Caller<ExecutionContext>, buffer_ptr: i32) -> Result<(), Error> {
    let calldata = caller.data().env().job_request().data.clone();
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &calldata)?;
    Ok(())  // TODO: remove?
}

fn on_chain_id(caller: Caller<ExecutionContext>) -> i64 {
    let context = caller.data();
    if let Some(id) = context.env().job_request().on_chain_id.clone() {
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
    let address = caller.data().env().caller_wallet().address().to_string();
    ic_cdk::println!("EVM caller wallet address: {}", address);
    write_utf16_string(&mut caller, &address, buffer_ptr)
}

/// Transfers the specified amount of gas tokens to the caller wallet.
/// This operates on the calling chain, so it only works if the Frosty Function was invoked from an EVM chain.
/// 
/// TODO: Offer a general transfer_gas function to any address. We'd just need to handle
/// replacing failing transactions to not cause all future transaction to get stuck.
fn evm_caller_wallet_deposit(mut caller: Caller<ExecutionContext>, amount: u64, promise_id: i32) -> Result<(), Error> {
    let evm_chain = match &caller.data().env().job_request().chain {
        Chain::Evm(id) => id.clone(),
        _ => return Err(Error::new("CallerWallet.deposit can only be used on EVM chains".to_string())),
    };
    let wallet = caller.data().env().caller_wallet();
    
    caller.data_mut().queue_task(
        promise_id,
        format!("CallerWallet.deposit({})", amount),
        // TODO: Move the messy parts into a spawn function.
        Box::pin(async move {
            // TODO: Check gas balance first.
            let tx = transfer_funds(evm_chain, wallet.address(), amount).await?;
            // TODO: In order to add to the execution logs, we'll need to store the execution object
            // and context on the heap and manually delete it after execution. We'll need some
            // cleanup mechanism for executions that are stale / paniced.
            // Alternatively, maybe just have a synchronous callback that's called after
            // the future completed already, so that we can do some logging etc.
            ic_cdk::println!("[#{}] Sent transaction with hash: 0x{}", promise_id, tx);
            // TODO: Do add the transaction to the execution logs.
            Ok(tx.as_slice().into())
        })
    );
    Ok(())
}

/// Signs a EIP-191 message.
// TOOD: Also expose lower level sign_hash to sign arbitrary hashes.  
fn evm_caller_wallet_sign_message(mut caller: Caller<ExecutionContext>, message_ptr: i32, promise_id: i32) -> Result<(), Error> {
    let message = read_buffer(&caller, message_ptr, BUFFER_MAX_LEN)?;
    let wallet = caller.data().env().caller_wallet();
    // TODO: Spawn rather than queue task
    caller.data_mut().queue_task(
        promise_id,
        format!("CallerWallet.signMessage(0x{})", clip_string(&hex::encode(&message), 100)),
        Box::pin(async move {
            let sig = wallet.sign_message(message.as_ref()).await
                .map_err(|e| format!("Failed to sign message: {}", e))?;
            ic_cdk::println!("Signed message length in bytes: {}", sig.as_bytes().len());
            Ok(sig.as_bytes().into())
        }) 
    );
    Ok(())
}

fn evm_chain_id(caller: Caller<ExecutionContext>) -> u64 {
    match &caller.data().env().job_request().chain  {
        Chain::Evm(id) => crate::evm::evm_chain_id(id.clone()),
        _ => 0,
    }
}

fn ic_raw_rand(mut caller: Caller<ExecutionContext>, promise_id: i32) -> Result<(), Error> {
    caller.data_mut().queue_task(
        promise_id,
        "Retrieve verifiable randomness".to_string(),
        Box::pin(async {
            ic_cdk::management_canister::raw_rand().await
                .map_err(|e| format!("Failed to get raw_rand: {}", e))
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
    let shared_buffer = caller.data_mut().commit_context().shared_buffer.clone();
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
    let buf_len = u32::from_le_bytes(buf_len) as usize;
    if buf_len > max_len {
        return Err(Error::new(format!("Buffer length {} exceeds maximum allowed {}", buf_len, max_len)));
    }

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

fn clip_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut clipped = s[..max_len - 3].to_string();
        clipped.push_str("...");
        clipped
    }
}
