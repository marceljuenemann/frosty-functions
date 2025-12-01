import { Promise } from "./promise";
import { SharedPromise } from "./internal/async";

export enum EvmChain {
    EthereumMainnet = 1,
    EthereumSepolia = 11155111,
    ArbitrumOne = 42161,
    ArbitrumSepolia = 421614,
    Localhost = 31337,
}

/**
 * The EIP-155 chain ID of the chain that invoked this Frosty Function,
 * or zero if invoked from a non-EVM chain.
 */
@lazy
export const CALLING_CHAIN_ID: u64 = callingChainId();

@lazy
export const CALLING_CHAIN_NAME: string = chainName(CALLING_CHAIN_ID);

@external("❄️", "evm_chain_id")
declare function callingChainId(): u64;

/**
 * Returns a user-friendly name for the given EVM chain ID.
 */
export function chainName(chainId: u64): string {
  switch (chainId) {
    case EvmChain.EthereumMainnet: return "Ethereum Mainnet";
    case EvmChain.EthereumSepolia: return "Ethereum Sepolia Testnet";
    case EvmChain.ArbitrumOne: return "Arbitrum One";
    case EvmChain.ArbitrumSepolia: return "Arbitrum Sepolia Testnet";
    case EvmChain.Localhost: return "Localhost EVM Node";
    default: return "EVM Chain ID " + chainId.toString();
  }
}

/**
 * An Ethereum Wallet that support signing arbitrary messages.
 * 
 * Currently this class only implements the `forCaller()` method to
 * retrieve the wallet for the caller that invoked the current Frosty Function.
 */
export class Wallet {
  private constructor(
    readonly address: string
  ) {}

  static forCaller(): Wallet {
    return new Wallet("test")
  }
}

@external("❄️", "evm_caller_wallet_address")
declare function callerWalletAddress(bufferPtr: i32): void;

/**
 * Submits a transaction to the EVM chain that invoked this Frosty Function.
 * 
 * The callback will be routed through the Frosty Function bridge contract
 * and call into the contract that called `invokeFunction`, unless it was
 * called by an external account.
 * 
 * Both the amount specified and the gas costs for the transaction will be
 * deducted from the gas of the current Frosty Function execution.
 * 
 * @param data arbitrary calldata to include in the callback
 * @param amount amount of native currency to include in the callback
 */
// TODO: Support amounts larger than 2^64 (which is around 18 ETH).
// TODO: Decide whether to still build a callback or only support the
// wallet method? Potential issue is that it requires an additional transaction,
// although not if the caller takes care of funding it properly beforehand.
/*
export function callback(data: ArrayBuffer, amount: u64): Promise<ArrayBuffer> {
  // TODO: Actually pass data and amount.
  // TODO: Have a reasonable return value.
  let promise = new SharedPromise();
  __evm_callback(promise.id, changetype<i32>(data), amount);
  return promise;
}

@external("❄️", "evm_callback")
declare function __evm_callback(promiseId: i32, dataPtr: i32, amount: u64): void;
*/
