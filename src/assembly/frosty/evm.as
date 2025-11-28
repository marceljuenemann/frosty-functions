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
