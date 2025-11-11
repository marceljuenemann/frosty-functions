use evm_rpc_client::CandidResponseConverter;
use evm_rpc_client::EvmRpcClient;
use evm_rpc_client::IcRuntime;
use evm_rpc_client::NoRetry;
use evm_rpc_types::BlockTag;
use evm_rpc_types::Hex20;
use evm_rpc_types::RpcServices;


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
    let result = client
        .get_logs(filter)
        .send()
        .await;

    // Format result as JSON-like string
    Ok(format!("{:?}", result))
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
