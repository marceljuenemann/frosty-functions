use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::Filter;
use alloy::rpc::types::Log;
use alloy::sol;
use alloy::sol_types::SolEvent;
use alloy::transports::icp::{L2MainnetService, RpcApi, RpcService};
use evm_rpc_types::Nat256;

use crate::chain::Chain;
use crate::chain::EvmChain;
use crate::evm::FrostyBridge::FunctionInvoked;
use crate::runtime::JobRequest;
use crate::storage::create_job;

// TODO: Move this all to crosschain module.

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
        .filter(|request| {
            if !create_job(request.clone()) {
                ic_cdk::println!("Job with ID {:?} on Chain {:?} already existed.", request.on_chain_id, chain);
                return false;
            }
            true
        })
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
        function_hash: event.inner.functionId.0.to_vec(),
        data: event.inner.data.data.to_vec(),
        gas_payment: Nat256::from_be_bytes(event.inner.gasPayment.to_be_bytes()),
    };
    if job.block_hash.is_none() || job.block_number.is_none() || job.transaction_hash.is_none() {
        return Err("Missing block hash, block number or transaction hash in event".to_string());
    }
    Ok(job)
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

// TODO: Move this into a config file. Maybe make it shareable with frontend as well?
fn bridge_address(chain: &EvmChain) -> Address{
    match chain {
        EvmChain::ArbitrumOne => "0xe712A7e50abA019A6d225584583b09C4265B037B",
        EvmChain::ArbitrumSepolia => "0xcAcbb4E46F2a68e3d178Fb98dCaCe59d12d54CBc",
        EvmChain::Localhost => "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"
    }.parse().unwrap()
}
