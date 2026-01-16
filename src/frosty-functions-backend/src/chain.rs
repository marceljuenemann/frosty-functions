
use candid::{CandidType};
use evm_rpc_types::{Hex20};
use serde::Deserialize;

#[derive(Debug, Clone, Hash, Ord, PartialOrd, PartialEq, Eq, CandidType, Deserialize)]
pub enum Chain {
    Evm(EvmChain)
}

impl Chain {
    pub fn is_testnet(&self) -> bool {
        match self {
            Chain::Evm(evm_chain) => evm_chain.is_testnet(),
        }
    }
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, PartialEq, Eq, CandidType, Deserialize)]
pub enum EvmChain {
    ArbitrumOne,
    ArbitrumSepolia,
    Localhost
}

impl EvmChain {
    pub fn is_testnet(&self) -> bool {
        match self {
            EvmChain::ArbitrumSepolia => true,
            EvmChain::ArbitrumOne => false,
            EvmChain::Localhost => true,
        }
    }

    pub fn tmp_gas_price(&self) -> u64 {
        // TODO: Replace with actual gas price fetching logic (somewhere else probably).
        match self {
            EvmChain::ArbitrumOne => 20_000_000,
            EvmChain::ArbitrumSepolia => 20_000_000,
            EvmChain::Localhost => 20_000_000,
        }
    }
}

/// A generic address type that can represent addresses from different blockchain types.
#[derive(Debug, Clone, candid::CandidType, serde::Serialize, serde::Deserialize)]
pub enum Address {
    EvmAddress(Hex20)
} 
