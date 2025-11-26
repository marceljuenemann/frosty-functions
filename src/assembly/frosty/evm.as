export enum EvmChain {
    EthereumMainnet = 1,
    EthereumSepolia = 11155111,
    ArbitrumOne = 42161,
    ArbitrumSepolia = 421614,
    Localhost = 31337,
}

/**
 * If the function was invoked from an EVM-compatible chain, returns the
 * EIP-155 chain ID. Returns zero for non-EVM chains.
 */
@external("❄️", "evm_chain_id")
export declare function evmChainId(): u64;

/**
 * Returns a user-friendly name for the given EVM chain ID.
 */
export function evmChainName(chainId: u64): string {
  switch (chainId) {
    case EvmChain.EthereumMainnet: return "Ethereum Mainnet";
    case EvmChain.EthereumSepolia: return "Ethereum Sepolia Testnet";
    case EvmChain.ArbitrumOne: return "Arbitrum One";
    case EvmChain.ArbitrumSepolia: return "Arbitrum Sepolia Testnet";
    case EvmChain.Localhost: return "Localhost EVM Node";
    default: return "EVM Chain ID " + chainId.toString();
  }
}
