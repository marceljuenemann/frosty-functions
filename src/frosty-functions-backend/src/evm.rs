use alloy::eips::BlockNumberOrTag;
use alloy::network::TransactionBuilder;
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::primitives::TxHash;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::Filter;
use alloy::rpc::types::Log;
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::SolEvent;
use alloy::transports::icp::{L2MainnetService, RpcApi, RpcService};
use evm_rpc_types::Nat256;

use crate::chain::Chain;
use crate::chain::EvmChain;
use crate::evm::FrostyBridge::FunctionInvoked;
use crate::job::JobRequest;
use crate::state::read_state;
use crate::storage::create_job;

sol! {
    #[sol(rpc)]
    "../../contracts/Bridge.sol"
}

/// Creates jobs from log events in the specified block.
pub async fn index_block(chain: &EvmChain, block_number: u64) -> Result<Vec<JobRequest>, String> {
    // TODO: Configure response size, use multiple providers etc.
    let config = alloy::transports::icp::IcpConfig::new(rpc_service(&chain));
    let provider = ProviderBuilder::new().on_icp(config);
    let filter = Filter::new()
        .address(bridge_address(chain))
        .event(FrostyBridge::FunctionInvoked::SIGNATURE)
        .from_block(BlockNumberOrTag::Number(block_number))
        .to_block(BlockNumberOrTag::Number(block_number));
    let job_ids = provider
        .get_logs(&filter)
        .await
        .map_err(|e: alloy::transports::RpcError<alloy::transports::TransportErrorKind>| format!("Failed to fetch Bridge events: {}", e))?
        .into_iter()
        // Create JobRequests from log events.
        .filter_map(|log| {
            let job = job_from_event(chain, log);
            if job.is_err() {
                ic_cdk::println!("ERROR: Failed to parse event from block {block_number} on chain {chain:?}: {}", job.as_ref().unwrap_err());
            }
            job.ok()
        })
        // Create job in storage (if it doesn't exist yet).
        .filter(|request| create_job(request.clone()))
        .collect();
    Ok(job_ids)
}

fn job_from_event(chain: &EvmChain, event: Log) -> Result<JobRequest, String> {
    let event = event.log_decode::<FunctionInvoked>()
        .map_err(|err| format!("Failed to decode log event {}", err))?;
    let job = JobRequest {
        chain: Chain::Evm(chain.clone()),
        block_hash: event.block_hash.map(|v| v.0.into()),
        block_number: event.block_number,
        transaction_hash: event.transaction_hash.map(|v| v.0.into()),
        on_chain_id: Some(Nat256::from_be_bytes(event.inner.jobId.to_be_bytes())),
        caller: crate::chain::Address::EvmAddress(event.inner.caller.0.0.into()),
        function_hash: <[u8; 32]>::from(event.inner.functionId),
        data: event.inner.data.data.to_vec(),
        gas_payment: Nat256::from_be_bytes(event.inner.gasPayment.to_be_bytes()),
    };
    if job.block_hash.is_none() || job.block_number.is_none() || job.transaction_hash.is_none() {
        return Err("Missing block hash, block number or transaction hash in event".to_string());
    }
    Ok(job)
}

/// Transfers funds from the canister's main EVM account to the specified address.
/// The nonce logic assumes that all transactions suceed, so callers should ensure
/// that enough gas is available on the account.
pub async fn transfer_funds( 
    chain: EvmChain,
    to_address: Address,
    amount: u64,
) -> Result<TxHash, String> {
    // TODO: Configure response size to save on cycles.
    let wallet = EthereumWallet::from(read_state(|s| s.main_signer.clone()));
    let config = alloy::transports::icp::IcpConfig::new(rpc_service(&chain));
    let provider = ProviderBuilder::new()
        // TODO: Always set 21000 as gas limit
        // TODO: Fetch base fee and cache it for some time.
        // TODO: Use our own NonceManager that persists nonces.
        // .with_gas_estimation()
        // .filler(NonceFiller::new(nonce_manager))
        .with_recommended_fillers()
        .wallet(wallet)
        .on_icp(config);

    let nonce = 1;  // TODO: increment.
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(amount))
        // .with_nonce(nonce)
        .with_chain_id(evm_chain_id(chain));

    let transaction_result = provider.send_transaction(tx.clone()).await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;
    Ok(transaction_result.tx_hash().clone())
}

fn rpc_service(evm_chain: &EvmChain) -> RpcService {
    // TODO: Fetch from multiple providers to ensure consistency.
    match evm_chain {
        EvmChain::Localhost => RpcService::Custom(RpcApi {
            url: "http://127.0.0.1:8545".to_string(),
            headers: None,
        }),
        EvmChain::ArbitrumSepolia => RpcService::Custom(RpcApi {
            url: "https://arbitrum-sepolia-rpc.publicnode.com".to_string(),
            headers: None,
        }),
        EvmChain::ArbitrumOne => RpcService::ArbitrumOne(L2MainnetService::Alchemy)
    }
}

pub fn evm_chain_id(chain: EvmChain) -> u64 {
    match chain {
        EvmChain::ArbitrumOne => 42161,
        EvmChain::ArbitrumSepolia => 421614,
        EvmChain::Localhost => 31337,
    }
}

// TODO: Move this into a config file. Maybe make it shareable with frontend as well?
fn bridge_address(chain: &EvmChain) -> Address{
    match chain {
        EvmChain::ArbitrumOne => "0xe712A7e50abA019A6d225584583b09C4265B037B",
        EvmChain::ArbitrumSepolia => "0xe712A7e50abA019A6d225584583b09C4265B037B",
        EvmChain::Localhost => "0x5FbDB2315678afecb367f032d93F642f64180aa3"
    }.parse().unwrap()
}
