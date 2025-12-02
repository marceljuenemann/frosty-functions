use alloy::{signers::icp::IcpSigner};

/// Subset of IcpSigner used as an identifier.
pub struct IcpSignerId {
    pub derivation_path: Vec<Vec<u8>>,
}

// TODO: Cache

/// Returns the signer for the canister's main EVM account (where gas is sent to).
/// Use read_state to access the cached signer instead of calling this repeatedly.
pub async fn main_signer() -> Result<IcpSigner, String> {
    signer(vec![]).await
}

/// Returns the wallet signer for the given chain and address.
/// 
/// Before signing any messages, it's crucial to verify that the owner of the address
/// authorized the message in some way, e.g. by invoking a Frosty Function on-chain.
pub async fn signer_for_address(address: &crate::chain::Address) -> Result<IcpSigner, String> {
    let address_bytes = match address {
        crate::chain::Address::EvmAddress(addr) => addr.clone(),
    };
    let derivation_path = vec![
        "❄️/wallet".as_bytes().to_vec(),
        address_bytes.as_ref().to_vec(),
    ];
    signer(derivation_path).await
}

async fn signer(derivation_path: Vec<Vec<u8>>) -> Result<IcpSigner, String> {
    IcpSigner::new(derivation_path, &key_id(), None)
        .await
        .map_err(|err| format!("Failed to create ICP signer: {}", err))
}


fn key_id() -> String {
    let dfx_network = option_env!("DFX_NETWORK").unwrap();
    match dfx_network {
        "local" => "dfx_test_key".to_string(),
        "ic" => "key_1".to_string(),
        _ => panic!("Unsupported DFX_NETWORK."),
    }
}
