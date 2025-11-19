use alloy_sol_types::Error;
use alloy_sol_types::abi::token::WordToken;
use evm_rpc_client::CandidResponseConverter;
use evm_rpc_client::EvmRpcClient;
use evm_rpc_client::IcRuntime;
use evm_rpc_client::NoRetry;
use evm_rpc_types::ConsensusStrategy;
use evm_rpc_types::{LogEntry};
use evm_rpc_types::Nat256;
use evm_rpc_types::{BlockTag, Hex20, RpcServices};
use alloy_sol_types::{sol, SolEvent};
use alloy_primitives::B256;

use crate::job::JobRequest;
use crate::chain::Address;

// Define the event with Alloy's sol! macro (must match Bridge.sol exactly)
sol! {
    #[derive(Debug)]
    event FunctionInvoked(address indexed caller, bytes32 indexed functionId, bytes data, uint256 gasPayment, uint256 jobId);
}

/// Fetches requested jobs from the EVM chain.
///
/// NOTE: This fetches jobs from unfinalized blocks that might be re-orged.
pub async fn fetch_jobs(chain_id: u64, contract_address: String, since_block: u64) -> Result<Vec<JobRequest>, String> {
    let client = create_client(chain_id);
    let address_hex: Hex20 = contract_address.parse().map_err(|e| format!("Invalid address: {}", e))?;
    let mut filter = evm_rpc_types::GetLogsArgs::from(vec![address_hex]);
    filter.from_block = Some(BlockTag::Number(Nat256::from(since_block)));
    filter.to_block = Some(BlockTag::Latest);

    // NOTE: Since we are fetching the latest block, inconsistent responses are more likely,
    // so using a 2 out of 3 consensus strategy seems important.
    match client.get_logs(filter).send().await {
        evm_rpc_types::MultiRpcResult::Consistent(Ok(events)) => {
          return jobs_from_events(chain_id, events);
        }
        evm_rpc_types::MultiRpcResult::Consistent(Err(err)) => {
            return Err(format!("EVM RPC error: {:?}", err));
        }
        evm_rpc_types::MultiRpcResult::Inconsistent(_) => {
            return Err("Inconsistent responses from EVM RPC providers".to_string());
        }
    }
}

fn jobs_from_events(chain_id: u64, events: Vec<LogEntry>) -> Result<Vec<JobRequest>, String> {
    let mut jobs = Vec::new();
    for event in events.iter().rev() {
        match decode_function_invocation(event) {
            Ok(func_invoked) => {
                let job_id_bytes = func_invoked.jobId.to_be_bytes::<32>();
                let on_chain_id = Nat256::from_be_bytes(job_id_bytes);
                
                let gas_payment_bytes = func_invoked.gasPayment.to_be_bytes::<32>();
                let gas_payment = Nat256::from_be_bytes(gas_payment_bytes);

                // Convert alloy Address (20 bytes) -> Hex20
                let caller = Hex20::from(func_invoked.caller.into_array());

                // Convert alloy FixedBytes<32> -> Hex32
                let function_hash = <[u8; 32]>::from(func_invoked.functionId);

                let job = JobRequest {
                    chain_id: format!("eip155:{}", chain_id),
                    block_hash: event.block_hash.clone(),
                    block_number: event.block_number.clone(),
                    transaction_hash: event.transaction_hash.clone(),
                    on_chain_id: Some(on_chain_id),
                    caller: Address::EvmAddress(caller),
                    function_hash,
                    data: func_invoked.data.to_vec(),
                    gas_payment,
                };
                jobs.push(job);
            }
            Err(err) => {
                return Err(format!("Failed to decode event: {:?}", err));
            }
        }
    }
    Ok(jobs)
}

fn decode_function_invocation(event: &LogEntry) -> Result<FunctionInvoked, Error> {
    let topics = event
        .topics
        .iter()
        .map(|hex32| WordToken(B256::from(hex32.as_array())))
        .collect::<Vec<_>>();
    FunctionInvoked::decode_raw_log(topics, event.data.as_ref(), true)
}

/// Creates an EVM RPC client for the specified chain ID.
/// 
/// All calls are sent to three different providers and a 2 out of 3 consensus is required.
fn create_client(chain_id: u64) -> EvmRpcClient<IcRuntime, CandidResponseConverter, NoRetry> {
    match chain_id {
        31337 => {
            // Use local EVM node for connecting to local chain.
            EvmRpcClient::builder_for_ic()
                .with_rpc_sources(RpcServices::Custom {
                    chain_id: 31337,
                    services: vec![evm_rpc_types::RpcApi {
                        url: "http://127.0.0.1:8545".to_string(),
                        headers: None,
                    }],
                })
                .build()
        }
        _ => {
            EvmRpcClient::builder_for_ic()
                .with_consensus_strategy(ConsensusStrategy::Threshold {
                    total: Some(3),
                    min: 2,
                })
                .build()
        }
    }
}
