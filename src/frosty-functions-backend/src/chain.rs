
use candid::{CandidType};
use evm_rpc_types::{Hex20};
use serde::{Deserialize, Serialize};

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
    pub fn chain_id(&self) -> u64 {
        match self {
            EvmChain::ArbitrumOne => 42161,
            EvmChain::ArbitrumSepolia => 421614,
            EvmChain::Localhost => 31337,
        }
    }

    pub fn is_testnet(&self) -> bool {
        match self {
            EvmChain::ArbitrumSepolia => true,
            EvmChain::ArbitrumOne => false,
            EvmChain::Localhost => true,
        }
    }
}

/// A generic address type that can represent addresses from different blockchain types.
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum Address {
    EvmAddress(Hex20)
} 

/// A caller identified by chain and address.
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct Caller {
    pub chain: crate::chain::Chain,
    pub address: crate::chain::Address,
}

impl Into<Vec<u8>> for Caller {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::new();
        match self.chain {
            crate::chain::Chain::Evm(evm_chain) => {
              let chain_id = evm_chain.chain_id();
              v.push(0u8);  // Chain type: EVM
              v.extend_from_slice(&chain_id.to_be_bytes());
            }
        }
        match self.address {
            crate::chain::Address::EvmAddress(evm_address) => {
                v.extend_from_slice(evm_address.as_ref());
            }
        }
        v
    }
}
