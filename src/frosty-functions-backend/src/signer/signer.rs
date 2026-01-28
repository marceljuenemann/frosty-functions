use ic_pub_key::{EcdsaKeyId, EcdsaPublicKeyArgs};

use crate::{chain::Caller, repository::FunctionId};
use alloy::{primitives::{Address, keccak256}, signers::{k256::{PublicKey, elliptic_curve}}};

pub trait Signer {
    fn public_key(&self) -> Result<Vec<u8>, String>;

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

    /// Derives a signer to be controlled by the Frosty Function with the given ID.
    pub fn for_function(function_id: FunctionId, derivation: Option<Vec<u8>>) -> Self {
        assert!(function_id.len() == 32, "Invalid function ID");
        let mut derivation_path = vec![
            "❄️/function".as_bytes().to_vec(),
            function_id,
        ];
        if derivation.is_some() {
          derivation_path.push(derivation.unwrap());
        }
        Self { derivation_path }
    }

    /// Derives a signer to be controlled by the given caller.
    pub fn for_caller(caller: Caller, derivation: Option<Vec<u8>>) -> Self {
        let mut derivation_path = vec![
            "❄️/caller".as_bytes().to_vec(),
            caller.into()
        ];
        if derivation.is_some() {
          derivation_path.push(derivation.unwrap());
        }
        Self { derivation_path }
    }
}

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
            key_id: EcdsaKeyId {
                curve: ic_pub_key::EcdsaCurve::Secp256k1,
                name: root_key_name,
            }
        }).map_err(|e| format!("Failed to derive public key: {:?}", e))?;
        Ok(public_key.public_key)
    }
}
