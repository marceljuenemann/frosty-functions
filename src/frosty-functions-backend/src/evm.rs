use alloy_primitives::U256;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::Error;
use alloy_sol_types::abi::token::WordToken;
use evm_rpc_client::AlloyResponseConverter;
use evm_rpc_client::EvmRpcClient;
use evm_rpc_client::NoRetry;
use evm_rpc_types::ConsensusStrategy;
use evm_rpc_types::{LogEntry};
use evm_rpc_types::Nat256;
use evm_rpc_types::{BlockTag, Hex20, RpcServices};
use alloy_sol_types::{sol, SolEvent};
use alloy_primitives::B256;
use ic_canister_runtime::IcRuntime;

use crate::chain::Chain;
use crate::chain::EvmChain;
use crate::job::JobRequest;
use crate::chain::Address;

// Define the event with Alloy's sol! macro (must match Bridge.sol exactly)
// TODO: Figure out how to import Bridge.sol without pulling in getrandom 
// #[sol(rpc)]
// "../../contracts/Bridge.sol"
sol! {
    #[derive(Debug)]
    event FunctionInvoked(address indexed caller, bytes32 indexed functionId, bytes data, uint256 gasPayment, uint256 jobId);
}

use alloy_rlp::Encodable;

pub async fn transfer_funds( 
    evm_chain: EvmChain,
    to_address: String,
    amount: u64,
) -> Result<(), String> {
    let client = create_client(evm_chain.clone());

    let mut transaction = TransactionRequest::default()
        .from("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".parse().unwrap())
        .to(to_address.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .value(U256::from(amount))
        // TODO: Determine these automatically.
        .max_priority_fee_per_gas(42u128)
        .max_fee_per_gas(54u128)
        .gas_limit(21000u64)
        // TODO: Set input
        .nonce(0);  // TODO: Use transaction count for this job (assuming job_id field is part of bridge call)
    transaction.chain_id = Some(evm_chain_id(evm_chain));
    ic_cdk::println!("TransactionRequest: {:?}", transaction);

    let tx = transaction.build_1559()
        .map_err(|e| format!("Failed to build transaction: {:?}", e))?;
    ic_cdk::println!("Unsinged tx: {:?}", tx);

    let mut buf = vec![];
    tx.encode(&mut buf);
    ic_cdk::println!("Unsinged tx: {:?}", buf);




    // Continue here: https://internetcomputer.org/docs/building-apps/chain-fusion/ethereum/using-eth/eth-dev-workflow
    // - Raw transaction bytes: https://alloy.rs/examples/transactions/encode_decode_eip1559
    // - Get a key. Probably use ic_evm_util
    // - Sign. See https://alloy.rs/examples/transactions/encode_decode_eip1559
    



   //  client.send_raw_transaction(transaction.into());



    Ok(())
}


/// Fetches requested jobs from the EVM chain.
///
/// NOTE: This fetches jobs from unfinalized blocks that might be re-orged.
pub async fn fetch_jobs(evm_chain: &EvmChain, contract_address: String, since_block: u64) -> Result<Vec<JobRequest>, String> {
    let client = create_client(evm_chain.clone());
    let address_hex: Hex20 = contract_address.parse().map_err(|e| format!("Invalid address: {}", e))?;
    let mut filter = evm_rpc_types::GetLogsArgs::from(vec![address_hex]);
    filter.from_block = Some(BlockTag::Number(Nat256::from(since_block)));
    filter.to_block = Some(BlockTag::Latest);
    filter.to_block = Some(BlockTag::Number(Nat256::from(since_block + 499))); // TODO: Remove hardcoding

    // NOTE: Since we are fetching the latest block, inconsistent responses are more likely,
    // so using a 2 out of 3 consensus strategy seems important.
    match client.get_logs(filter).send().await {
        evm_rpc_types::MultiRpcResult::Consistent(Ok(events)) => {
            Err("Not yet implemented".to_string())
            // TODO: Move to alloy types?
            // return jobs_from_events(evm_chain, events);
        }
        evm_rpc_types::MultiRpcResult::Consistent(Err(err)) => {
            return Err(format!("EVM RPC error: {:?}", err));
        }
        evm_rpc_types::MultiRpcResult::Inconsistent(_) => {
            return Err("Inconsistent responses from EVM RPC providers".to_string());
        }
    }
}

fn jobs_from_events(chain: &EvmChain, events: Vec<LogEntry>) -> Result<Vec<JobRequest>, String> {
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
                    chain: Chain::Evm(chain.clone()),
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
    FunctionInvoked::decode_raw_log(topics, event.data.as_ref())
}

/// Creates an EVM RPC client for the specified chain.
/// 
/// All calls are sent to three different providers and a 2 out of 3 consensus is required.
fn create_client(evm_chain: EvmChain) -> EvmRpcClient<IcRuntime, AlloyResponseConverter, NoRetry> {
    let mut builder = EvmRpcClient::builder_for_ic()
      .with_alloy()
      .with_rpc_sources(get_rpc_sources(&evm_chain));
    if evm_chain != EvmChain::Localhost {
        builder = builder.with_consensus_strategy(ConsensusStrategy::Threshold {
            total: Some(3),
            min: 2,
        });
    }
    builder.build()
}

fn get_rpc_sources(evm_chain: &EvmChain) -> RpcServices {
    match evm_chain {
        EvmChain::Localhost => RpcServices::Custom {
            chain_id: evm_chain_id(EvmChain::Localhost),
            services: vec![evm_rpc_types::RpcApi {
                url: "http://127.0.0.1:8545".to_string(),
                headers: None,
            }],
        },
        EvmChain::ArbitrumSepolia => RpcServices::Custom {
            chain_id: evm_chain_id(EvmChain::ArbitrumSepolia),
            services: vec![
                evm_rpc_types::RpcApi {
                    url: "https://arbitrum-sepolia-rpc.publicnode.com".to_string(),
                    headers: None,
                },
                evm_rpc_types::RpcApi {
                    url: "https://arbitrum-sepolia.drpc.org".to_string(),
                    headers: None,
                },
                evm_rpc_types::RpcApi {
                    url: "https://arbitrum-sepolia.gateway.tenderly.co".to_string(),
                    headers: None,
                },
            ],
        },
        EvmChain::ArbitrumOne => RpcServices::ArbitrumOne(None)
    }
}

pub fn evm_chain_id(chain: EvmChain) -> u64 {
    match chain {
        EvmChain::ArbitrumOne => 42161,
        EvmChain::ArbitrumSepolia => 421614,
        EvmChain::Localhost => 31337,
    }
}
