use async_trait::async_trait;

use crate::signer::{Signer, ThresholdSigner};

/// Unsafe signer implementation for simulation purposes only.
pub struct SimulationSigner {
    threshold_signer: ThresholdSigner
}

impl SimulationSigner {
    pub fn new(derivation_path: Vec<Vec<u8>>) -> Self {
        let threshold_signer = ThresholdSigner::new(derivation_path);
        Self { threshold_signer }
    }
}

#[async_trait(?Send)]
impl Signer for SimulationSigner {
    fn public_key(&self) -> Result<Vec<u8>, String> {
        self.threshold_signer.public_key()
    }

    async fn sign_with_ecdsa(&self, msg_hash: Vec<u8>) -> Result<Vec<u8>, String> {
        // TODO: Replace with actual signing in memory.
        let dummy_signature = [42u8; 64];
        Ok(dummy_signature.to_vec())
    }
}
