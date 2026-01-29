import { chainName } from "frosty/evm";
import { Hex } from "frosty/hex";

/**
 * The data payload passed when the Frosty Function was invoked.
 */
@lazy
export const CALLDATA = ((): Hex => {
  // Note that the compiler won't strip this out, even if CALLDATA
  // is never used. So we should prefer functions over constants for
  // constants less likely to be used.
  let buffer = new ArrayBuffer(CALLDATA_SIZE);
  __calldata(changetype<i32>(buffer));
  return Hex.wrapArrayBuffer(buffer);
})();

@external("❄️", "CALLDATA_SIZE")
declare const CALLDATA_SIZE: i32;

@external("❄️", "calldata")
declare function __calldata(buffer_ptr: i32): void;

/**
 * Job ID generated on the block chain that invoked this contract.
 * 
 * Note that this ID is only unique in the context of the calling chain.
 * Even then, you might observe multiple invocations with the same Job ID
 * in case blocks get re-orged. This function might return -1 in the future
 * when invoked off-chain.
 */
@lazy
export const JOB_ID = __on_chain_id();

@external("❄️", "on_chain_id")
declare function __on_chain_id(): u64;

/**
 * The EIP-155 chain ID of the chain that invoked this Frosty Function,
 * or zero if invoked from a non-EVM chain.
 */
@lazy
export const CALLING_CHAIN_ID: u64 = __evm_chain_id();

@lazy
export const CALLING_CHAIN_NAME: string = chainName(CALLING_CHAIN_ID);

@external("❄️", "evm_chain_id")
declare function __evm_chain_id(): u64;
