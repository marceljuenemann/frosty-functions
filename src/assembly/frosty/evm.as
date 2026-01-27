import { Promise, Done, DONE } from "./promise";
import { SharedPromise } from "./internal/async";

export enum EvmChain {
    EthereumMainnet = 1,
    EthereumSepolia = 11155111,
    ArbitrumOne = 42161,
    ArbitrumSepolia = 421614,
    Localhost = 31337,
}

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
export class EthWallet {

  // TODO: Move to Address class with a toString method
  address(): string {
    let buffer = new ArrayBuffer(42 * 2);  // 42 chars * 2 bytes / char
    __evm_caller_wallet_address(changetype<i32>(buffer));
    return String.UTF16.decode(buffer)
  }

  /**
   * Signs the given message according to EIP-191
   * 
   * sign_hash(keccak256(0x19 <0x45 (E)> <thereum Signed Message:\n" + len(message)> <data to sign>))
   * 
   * Use String.UTF8.encode or String.UTF16.encode to convert a string to an ArrayBuffer.
   */
  signMessage(message: ArrayBuffer): Promise<Uint8Array> {
    let promise = new SharedPromise();
    __evm_caller_wallet_sign_message(changetype<i32>(message), promise.id);
    return promise.map<Uint8Array>(buffer => Uint8Array.wrap(buffer));
  }

  static forCaller(): EthWallet {
    return new EthWallet()
  }
}

@external("❄️", "evm_caller_wallet_address")
declare function __evm_caller_wallet_address(bufferPtr: i32): void;

@external("❄️", "evm_caller_wallet_sign_message")
declare function __evm_caller_wallet_sign_message(messagePtr: i32, promiseId: i32): void;
