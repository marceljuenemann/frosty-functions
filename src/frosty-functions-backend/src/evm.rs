use evm_rpc_client::CandidResponseConverter;
use evm_rpc_client::EvmRpcClient;
use evm_rpc_client::IcRuntime;
use evm_rpc_client::NoRetry;
use evm_rpc_types::Hex;
use evm_rpc_types::LogEntry;
use evm_rpc_types::RpcError;
use evm_rpc_types::{BlockTag, Hex20, RpcServices};
use alloy_sol_types::{sol, SolEvent};
use alloy_primitives::{B256, Bytes};
use serde::Serialize;

// Define the event with Alloy's sol! macro (must match Bridge.sol exactly)
sol! {
    #[derive(Debug)]
    event FunctionInvoked(address indexed caller, bytes32 indexed functionId, bytes data, uint256 gasPayment, uint256 jobId);
}

#[derive(Debug, Serialize)]
struct DecodedEvent {
    caller: String,
    function_id: String,
    data: String,
    gas_payment_wei: String,
    job_id: String,
}

pub async fn tmp_get_logs(contract_address: String) -> Result<String, String> {
    let client = create_client(31337);

    // Convert address string to Hex20
    let address_hex: Hex20 = contract_address
        .parse()
        .map_err(|e| format!("Invalid contract address: {:?}", e))?;

    // Build GetLogsArgs using the From implementation (pass iterator of addresses)
    let mut filter = evm_rpc_types::GetLogsArgs::from(vec![address_hex]);
    filter.from_block = Some(BlockTag::Earliest);
    filter.to_block = Some(BlockTag::Latest);  // TODO: not safe?

    // Call getLogs and send the request
    // TODO: Look in to the cycles consumption of this call
    let result = client
        .get_logs(filter)
        .send()
        .await;

    // Collect decoded events (best-effort even if providers disagree)
    match result {
        evm_rpc_types::MultiRpcResult::Consistent(Ok(events)) => {
          // collect_decoded(res, &mut decoded);

          for event in events {
              decode_invocation_event(&event);
          }
          // res.map(op)
        
        
        
        }


        // TODO: Handle as error
        _ => {
            return Err("Failed to get events from EVM RPC".to_string());
        }
    }

    Ok(String::from("TBD"))

    //serde_json::to_string(&decoded).map_err(|e| format!("Serialize error: {e}"))
}

fn decode_invocation_event(event: &LogEntry) -> Option<DecodedEvent> {
    ic_cdk::println!("Decoding event: {:?}", event);

    ic_cdk::println!("Data event: {:?}", event.data);
    // Convert the hex data in the log entry to Bytes using the local helper.
    let data_bytes = event.data.as_ref();
    let decoded = FunctionInvoked::abi_decode_data(data_bytes, true);
    ic_cdk::println!("Decoded data: {:?}", decoded);

    //let topics = FunctionInvoked::decode_log(event, true);
    //ic_cdk::println!("Decoded topics: {:?}", topics);

    None
}

fn hex_to_b256(s: &str) -> Option<B256> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 { return None; }
    let mut bytes = [0u8; 32];
    hex::decode_to_slice(s, &mut bytes).ok()?;
    Some(B256::from(bytes))
}

fn create_client(chain_id: u64) -> EvmRpcClient<IcRuntime, CandidResponseConverter, NoRetry> {
    match chain_id {
        31337 => {
            // Use local EVM node for connecting to local chain.
            return EvmRpcClient::builder_for_ic()
                .with_rpc_sources(RpcServices::Custom {
                    chain_id: 31337,
                    services: vec![evm_rpc_types::RpcApi {
                        url: "http://127.0.0.1:8545".to_string(),
                        headers: None,
                    }],
                })
                .build();
        }
        _ => return EvmRpcClient::builder_for_ic().build(),
    };
}
