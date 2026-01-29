use ic_cdk::management_canister::{EcdsaCurve, EcdsaKeyId, SignWithEcdsaArgs, sign_with_ecdsa};
use ic_pub_key::{EcdsaPublicKeyArgs};

use crate::{chain::Caller, repository::FunctionId};
use alloy::{primitives::{Address, keccak256}, signers::{k256::{PublicKey, elliptic_curve}}};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Signer {

    fn public_key(&self) -> Result<Vec<u8>, String>;

    // Result is the concatenation of the SEC1 encodings of the two values r and s.
    async fn sign_with_ecdsa(&self, msg_hash: Vec<u8>) -> Result<Vec<u8>, String>;

    // TODO: Patch ic_alloy to make address_for_public_key synchronous. 
    fn eth_address(&self) -> Result<Address, String> {
        let key: PublicKey = PublicKey::from_sec1_bytes(self.public_key()?.as_ref())
            .map_err(|e| e.to_string())?;
        let point = elliptic_curve::sec1::ToEncodedPoint::to_encoded_point(&key, false);
        let point_bytes = point.as_bytes();
        let hash = keccak256(&point_bytes[1..]);
        Ok(Address::from_slice(&hash[12..32]))
    }
}

pub struct ThresholdSigner {
    derivation_path: Vec<Vec<u8>>,
}

impl ThresholdSigner {
    pub fn new(derivation_path: Vec<Vec<u8>>) -> Self {
        Self { derivation_path }
    }
}

#[async_trait(?Send)]
impl Signer for ThresholdSigner {
    /// Derives the public key for this signer.
    ///
    /// The derivation is performed "offline" with hard coded root keys
    /// rather than by calling the management canister.
    fn public_key(&self) -> Result<Vec<u8>, String> {
        let dfx_network = option_env!("DFX_NETWORK").unwrap();
        let root_key_name = match dfx_network {
            "ic" => "key_1".to_string(),
            "local" => "pocketic_key_1".to_string(),
            _ => panic!("Unsupported DFX_NETWORK."),
        };
        let public_key = ic_pub_key::derive_ecdsa_key(&EcdsaPublicKeyArgs {
            canister_id: Some(ic_cdk::api::canister_self()),
            derivation_path: self.derivation_path.clone(),
            key_id: ic_pub_key::EcdsaKeyId {
                curve: ic_pub_key::EcdsaCurve::Secp256k1,
                name: root_key_name,
            }
        }).map_err(|e| format!("Failed to derive public key: {:?}", e))?;
        Ok(public_key.public_key)
    }

    async fn sign_with_ecdsa(&self, msg_hash: Vec<u8>) -> Result<Vec<u8>, String> {
        let response = sign_with_ecdsa(&SignWithEcdsaArgs {
            message_hash: msg_hash,
            derivation_path: self.derivation_path.clone(),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: "key_1".to_string(),
            }
        })
        .await
        .map_err(|e| format!("Failed to sign with ECDSA: {:?}", e))?;
        Ok(response.signature)
    }
}

/// Derives a signer to be controlled by the Frosty Function with the given ID.
pub fn derivation_path_for_function(function_id: FunctionId, derivation: Option<Vec<u8>>) -> Vec<Vec<u8>> {
    assert!(function_id.len() == 32, "Invalid function ID");
    let mut derivation_path = vec![
        "❄️/function".as_bytes().to_vec(),
        function_id,
    ];
    if derivation.is_some() {
        derivation_path.push(derivation.unwrap());
    }
    derivation_path
}

/// Derives a signer to be controlled by the given caller.
pub fn derivation_path_for_caller(caller: Caller, derivation: Option<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut derivation_path = vec![
        "❄️/caller".as_bytes().to_vec(),
        caller.into()
    ];
    if derivation.is_some() {
        derivation_path.push(derivation.unwrap());
    }
    derivation_path
}
