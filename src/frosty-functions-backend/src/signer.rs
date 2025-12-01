use alloy::{primitives::Address, signers::icp::IcpSigner};

use crate::chain::Chain;

/// Subset of IcpSigner used as an identifier.
pub struct IcpSignerId {
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: String,
}

// TODO: Cache


/// Returns the wallet signer for the given chain and address.
/// 
/// Before signing any messages, it's crucial to verify that the owner of the address
/// authorized the message in some way, e.g. by invoking a Frosty Function on-chain.
pub async fn signer_for_address(chain: &Chain, address: &crate::chain::Address) -> Result<IcpSigner, String> {
    let address_bytes = match address {
        crate::chain::Address::EvmAddress(addr) => addr.clone(),
    };
    let derivation_path = vec![
        "❄️/wallet".as_bytes().to_vec(),
        address_bytes.as_ref().to_vec(),
    ];
    IcpSigner::new(derivation_path, &public_key_id(chain), None)
        .await
        .map_err(|err| format!("Failed to create ICP signer: {}", err))
}

fn public_key_id(chain: &Chain) -> String {
    /*
        let dfx_network = option_env!("DFX_NETWORK").unwrap();
    match dfx_network {
        "local" => "dfx_test_key".to_string(),
        "ic" => "key_1".to_string(),
        _ => panic!("Unsupported network."),
    }
     */
    // Only three keys are available on ICP, see
    // https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa#signing-messages

    return if chain.is_testnet() { "test_key_1".to_string() } else { "key_1".to_string() };
}
