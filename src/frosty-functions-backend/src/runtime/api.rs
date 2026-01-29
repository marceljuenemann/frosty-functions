use std::cell::{Ref, RefCell, RefMut};
use std::env;
use std::rc::Rc;

use alloy::primitives::{Address, keccak256};
use ic_stable_structures::Storable;
use wasmi::{Caller, Error, Func, Global, Linker, Memory, Mutability, Store, Val, errors::LinkerError};
use crate::runtime::{LogEntry, LogType, RuntimeEnvironment, job};
use crate::signer::{Signer, ThresholdSigner};
use crate::{Chain};
use crate::runtime::runtime::{ExecutionContext};

/// The maximum length of data that can be passed to/from the guest.
// TODO: Consider increasing if there is a use case.
const BUFFER_MAX_LEN: usize = 10_000_000;

/// The maximum length of console log messages.
const CONSOLE_LOG_MAX_LEN: usize = 10_000;

// Constants used in simulations.
const SIMULATION_ADDRESS: &str = "0x1234567890abcdef1234567890abcdef12345678";

const CYCLES_RAW_RAND: u64 = 5_400_000;

// Signing on ICP is quite expensive, see https://docs.internetcomputer.org/references/t-sigs-how-it-works/#fees-for-the-t-ecdsa-production-key
// TODO: Add costs for the cansiter call, which depends on the length of the message.
const CYCLES_SIGN_MESSAGE: u64 = 26_153_846_153;
const CYCLES_EVM_RPC_CALL: u64 = 1_000_000_000;  // TODO: Calculate exact value.

const SIGNER_FOR_CALLER: i32 = 0;
const SIGNER_FOR_FUNCTION: i32 = 1;

// TODO: Simplify this. Get rid of all the macros.
// TODO: Maybe move the Rc into an ExecutionContextInner.
pub type Ctx = Rc<RefCell<ExecutionContext>>;

macro_rules! ctx {
    ($caller:expr) => {
        $caller.data_mut().borrow_mut() as std::cell::RefMut<'_, ExecutionContext>
    };
}

macro_rules! env {
    ($caller:expr) => {
        ctx!($caller).env()
    };
}

macro_rules! job {
    ($caller:expr) => {
        env!($caller).job_request()
    }
}

/// Registers all constants into the given linker.
pub fn register_constants(linker: &mut Linker<Ctx>, store: &mut Store<Ctx>) -> Result<(), LinkerError> {
    let calldata_size = env!(store).job_request().data.len() as i32;
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
pub fn register_host_functions(linker: &mut Linker<Ctx>, store: &mut Store<Ctx>) -> Result<(), LinkerError> {
    linker.define("env", "abort", Func::wrap(&mut *store, abort_host))?;
    linker.define("env", "console.log", Func::wrap(&mut *store, console_log))?;
    linker.define("env", "seed", Func::wrap(&mut *store, seed))?;

    register!(calldata, linker, store);
    register!(copy_shared_buffer, linker, store);
    register!(on_chain_id, linker, store);

    register!(signer_public_key, linker, store);
    register!(signer_eth_address, linker, store);
    //register!(evm_caller_wallet_sign_message, linker, store);
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
fn console_log(mut caller: Caller<Ctx>, message_ptr: i32) {
    let message = read_utf16_string(&caller, message_ptr, CONSOLE_LOG_MAX_LEN)
        // TODO: Return error?
        .unwrap_or_else(|e| format!("(failed to read log message: {})", e));
    ctx!(caller).commit_context().logs.push(LogEntry { level: LogType::Default, message: message.clone() });
    // TODO: Charge cycles for logs storage.
    // TODO: Limit log size?
}

/// Writes the calldata into the provided buffer, which is expected to be of CALLDATA_SIZE.
fn calldata(mut caller: Caller<Ctx>, buffer_ptr: i32) -> Result<(), Error> {
    let calldata = job!(caller).data.clone();
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &calldata)?;
    Ok(())  // TODO: remove?
}

fn on_chain_id(mut caller: Caller<Ctx>) -> i64 {
    if let Some(id) = job!(caller).on_chain_id.clone() {
        // TODO: Proper error handling for overflows.
        let id: u64 = id.to_string().parse().unwrap();
        id as i64
    } else {
        -1
    }
}

fn signer_public_key(mut caller: Caller<Ctx>, signer_type: i32, signer_derivation: i32, buffer_ptr: i32) -> Result<(), Error> {
    let signer = get_signer(&caller, signer_type, signer_derivation)?;
    let public_key = signer.public_key().map_err(|e| Error::new(e))?;
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &public_key)?;
    Ok(())
}

fn signer_eth_address(mut caller: Caller<Ctx>, signer_type: i32, signer_derivation: i32, buffer_ptr: i32) -> Result<(), Error> {
    let signer = get_signer(&caller, signer_type, signer_derivation)?;
    let address = signer.eth_address().map_err(|e| Error::new(e))?;
    get_memory(&caller).write(&mut caller, buffer_ptr as usize, &address.to_bytes())?;
    Ok(())
}

fn get_signer(caller: &Caller<Ctx>, signer_type: i32, signer_derivation: i32) -> Result<Box<dyn Signer>, Error> {
    let derivation = if signer_derivation != 0 {
        Some(read_buffer(&caller, signer_derivation, 1024)?)
    } else {
        None
    };
    let job = caller.data().borrow().env().job_request().clone();
    let signer = match signer_type {
        SIGNER_FOR_CALLER => {
            ThresholdSigner::for_caller(crate::chain::Caller {
                chain: job.chain,
                address: job.caller,
            }, derivation)
        },
        SIGNER_FOR_FUNCTION => {
            ThresholdSigner::for_function(job.function_hash, derivation)
        },
        _ => return Err(Error::new(format!("Invalid signer type: {}", signer_type))),
    };
    Ok(Box::new(signer))
}

/*
/// Writes the caller's wallet address as a UTF-16LE string into the provided buffer.
/// The buffer is expected to be large enough to hold the address string.
fn evm_caller_wallet_address(mut caller: Caller<Ctx>, buffer_ptr: i32) -> Result<(), Error> {
    // TODO: Make async.
    // TODO: Charge cycles for the inter-canister call.
    let address: Address = if env!(caller).is_simulation() {
        SIMULATION_ADDRESS.parse().unwrap()
    } else {
        env!(caller).caller_wallet().unwrap().address()
    };
    write_utf16_string(caller, &address.to_string(), buffer_ptr)
}

/// Signs a EIP-191 message.
// TOOD: Also expose lower level sign_hash to sign arbitrary hashes.  
fn evm_caller_wallet_sign_message(mut caller: Caller<Ctx>, message_ptr: i32, promise_id: i32) -> Result<(), Error> {
    ctx!(caller).charge_cycles(CYCLES_SIGN_MESSAGE)?;
    let message = read_buffer(&caller, message_ptr, 100_000)?;
    let mut ctx = ctx!(caller);
    let wallet = ctx.env().caller_wallet();
    // TODO: Refactor this. Probably create an AsyncContext that is passed to the closure.
    let is_simulation = ctx.env().is_simulation();
    ctx.queue_task(
        promise_id,
        format!("CallerWallet.signMessage(0x{})", clip_string(&hex::encode(&message), 100)),
        Box::pin(async move {
            if is_simulation {
                let dummy_signature = [42u8; 65];
                return Ok(dummy_signature.into());
            } else {
                let sig = wallet.unwrap().sign_message(message.as_ref()).await
                    .map_err(|e| format!("Failed to sign message: {}", e))?;
                Ok(sig.as_bytes().into())
            }
        }) 
    );
    Ok(())
}
*/

fn evm_chain_id(mut caller: Caller<Ctx>) -> u64 {
    match job!(caller).chain.clone() {
        Chain::Evm(id) => id.chain_id(),
        _ => 0,
    }
}

fn ic_raw_rand(mut caller: Caller<Ctx>, promise_id: i32) -> Result<(), Error> {
    ctx!(caller).charge_cycles(CYCLES_RAW_RAND)?;
    let is_simulation = env!(caller).is_simulation();
    ctx!(caller).queue_task(
        promise_id,
        "Retrieve verifiable randomness".to_string(),
        // TODO: Turn into a FnOnce that receives a ctx.
        Box::pin(async move {
            if !is_simulation {
                ic_cdk::management_canister::raw_rand().await
                    .map_err(|e| format!("Failed to get raw_rand: {}", e))
            } else {
                let bytes = ic_cdk::api::time().to_le_bytes();
                let rand = keccak256(bytes);
                Ok(rand.to_vec())
            }
        })
    );
    Ok(())
}

// Reads a UTF-16LE encoded string from the guest memory at the given pointer.
// TODO: What about error handling? Host function should be able to return Result as well?
fn read_utf16_string(caller: &wasmi::Caller<Ctx>, str_ptr: i32, max_len: usize) -> Result<String, Error> {
    let bytes = read_buffer(caller, str_ptr, max_len * 2)?;
    let mut u16s = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        u16s.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16(&u16s)
        .map_err(|e| Error::new(format!("Invalid UTF-16 string: {}", e)))
}

/// Writes the given string as UTF-16LE into the guest memory at the given pointer.
fn write_utf16_string(mut caller: Caller<Ctx>, str: &String, buffer_ptr: i32) -> Result<(), Error> {
    let bytes: Vec<u8> = str.encode_utf16().flat_map(|unit| unit.to_le_bytes()).collect();
    ic_cdk::println!("Writing UTF-16. length: {}", bytes.len());
    get_memory(&caller).write(caller, buffer_ptr as usize, &bytes)?;
    Ok(())
}

/// Copies the shared buffer into the guest memory at the given pointer.
/// The guest is expected to have allocated a buffer of the same size.
fn copy_shared_buffer(mut caller: Caller<Ctx>, buffer_ptr: i32) -> Result<(), Error> {
    let shared_buffer = ctx!(caller).commit_context().shared_buffer.clone();
    get_memory(&caller).write(caller, buffer_ptr as usize, &shared_buffer)?;
    Ok(())
}

/// Reads a buffer from the guest memory at the given pointer.
/// The length of the buffer is presumed to be in the first 4 bytes before the pointer.
fn read_buffer(caller: &Caller<Ctx>, ptr: i32, max_len: usize) -> Result<Vec<u8>, Error> {
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

fn get_memory(caller: &Caller<Ctx>) -> Memory {
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
