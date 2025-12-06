use alloy::eips::BlockNumberOrTag;
use alloy::network::TransactionBuilder;
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::primitives::TxHash;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use alloy::rpc::types::Filter;
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::SolEvent;
use alloy::transports::icp::{L2MainnetService, RpcApi, RpcService};
use evm_rpc_types::Nat256;

use crate::chain::EvmChain;
use crate::job::JobRequest;
use crate::state::read_state;

// Define the event with Alloy's sol! macro (must match Bridge.sol exactly)
// TODO: Figure out how to import Bridge.sol without pulling in getrandom 
// #[sol(rpc)]
// "../../contracts/Bridge.sol"
sol! {
//    #[derive(Debug)]
//    event FunctionInvoked(address indexed caller, bytes32 indexed functionId, bytes data, uint256 gasPayment, uint256 jobId);
    #[sol(rpc)]
    "../../contracts/Bridge.sol"
}

pub async fn index_block(chain: &EvmChain, block_number: u64) -> Result<Vec<Nat256>, String> {
    // TODO: Configure response size, use multiple providers etc.
    let config = alloy::transports::icp::IcpConfig::new(rpc_service(&chain));
    let provider = ProviderBuilder::new().on_icp(config);
    let filter = Filter::new()
        .address(bridge_address(chain))
        .event(FrostyBridge::FunctionInvoked::SIGNATURE)
        .from_block(BlockNumberOrTag::Number(block_number))
        .to_block(BlockNumberOrTag::Number(block_number));

    let logs = provider.get_logs(&filter).await
        .map_err(|e| format!("Failed to fetch Bridge events: {}", e))?;
    for log in logs.iter() {
        ic_cdk::println!("Found Bridge event: {:?}", log);
    }

    Err("Not yet implemented".to_string())
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
        .with_recommended_fillers()
//        .with_gas_estimation()
//        .filler(NonceFiller::new(nonce_manager))
        .wallet(wallet)
        .on_icp(config);

    let nonce = 1;  // TODO: increment.
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(amount))
//        .with_nonce(nonce)
        .with_chain_id(evm_chain_id(chain));

    let transaction_result = provider.send_transaction(tx.clone()).await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;
    Ok(transaction_result.tx_hash().clone())
}
    
/// Fetches requested jobs from the EVM chain.
///
/// NOTE: This fetches jobs from unfinalized blocks that might be re-orged.
pub async fn fetch_jobs(evm_chain: &EvmChain, contract_address: String, since_block: u64) -> Result<Vec<JobRequest>, String> {
    Err("Not yet implemented".to_string())
}

// fn jobs_from_events(chain: &EvmChain, events: Vec<LogEntry>) -> Result<Vec<JobRequest>, String> {
//     let mut jobs = Vec::new();
//     for event in events.iter().rev() {
//         match decode_function_invocation(event) {
//             Ok(func_invoked) => {
//                 let job_id_bytes = func_invoked.jobId.to_be_bytes::<32>();
//                 let on_chain_id = Nat256::from_be_bytes(job_id_bytes);
                
//                 let gas_payment_bytes = func_invoked.gasPayment.to_be_bytes::<32>();
//                 let gas_payment = Nat256::from_be_bytes(gas_payment_bytes);

//                 // Convert alloy Address (20 bytes) -> Hex20
//                 let caller = Hex20::from(func_invoked.caller.into_array());

//                 // Convert alloy FixedBytes<32> -> Hex32
//                 let function_hash = <[u8; 32]>::from(func_invoked.functionId);

//                 let job = JobRequest {
//                     chain: Chain::Evm(chain.clone()),
//                     block_hash: event.block_hash.clone(),
//                     block_number: event.block_number.clone(),
//                     transaction_hash: event.transaction_hash.clone(),
//                     on_chain_id: Some(on_chain_id),
//                     caller: crate::chain::Address::EvmAddress(caller),
//                     function_hash,
//                     data: func_invoked.data.to_vec(),
//                     gas_payment,
//                 };
//                 jobs.push(job);
//             }
//             Err(err) => {
//                 return Err(format!("Failed to decode event: {:?}", err));
//             }
//         }
//     }
//     Ok(jobs)
// }

// fn decode_function_invocation(event: &LogEntry) -> Result<FunctionInvoked, Error> {
//     let topics = event
//         .topics
//         .iter()
//         .map(|hex32| WordToken(B256::from(hex32.as_array())))
//         .collect::<Vec<_>>();
//     FunctionInvoked::decode_raw_log(topics, event.data.as_ref())
// }

/// Creates an EVM RPC client for the specified chain.
/// 
/// All calls are sent to three different providers and a 2 out of 3 consensus is required.
// fn create_client(evm_chain: EvmChain) -> EvmRpcClient<IcRuntime, AlloyResponseConverter, NoRetry> {
//     let mut builder = EvmRpcClient::builder_for_ic()
//       .with_alloy()
//       .with_rpc_sources(get_rpc_sources(&evm_chain));
//     if evm_chain != EvmChain::Localhost {
//         builder = builder.with_consensus_strategy(ConsensusStrategy::Threshold {
//             total: Some(3),
//             min: 2,
//         });
//     }
//     builder.build()
// }

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
