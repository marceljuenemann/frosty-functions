use std::{cell::RefCell, collections::HashMap};

use alloy::signers::icp::IcpSigner;

use crate::{chain::{Chain, ChainState}, signer::{IcpSignerId, main_signer}};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::new(None);
}

pub struct State {
    /// State specific to each chain.
    pub chains: HashMap<Chain, ChainState>,
    /// The signer for the main account of the canister (where gas is transferred to). 
    pub main_signer: IcpSigner,
    pub evm_caller_signers: HashMap<IcpSignerId, IcpSigner>
}

pub async fn init_state() {
    let main_signer = main_signer().await.unwrap();
    STATE.with_borrow_mut(|state| {
        if state.is_some() {
            // Already initialized. Might use for upgrades in the future though.
            return;
        }
        *state = Some(State {
            chains: HashMap::new(),
            main_signer,
            evm_caller_signers: HashMap::new()
        });
    });
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect("State is not initialized")))
}

pub fn mutate_state<F, R>(f: F) -> R where F: FnOnce(&mut State) -> R {
    STATE.with_borrow_mut(|s| f(s.as_mut().expect("State is not initialized")))
}

pub fn read_chain_state<R>(chain: &Chain, f: impl FnOnce(&ChainState) -> Result<R, String>) -> Result<R, String> {
    read_state(|state| {
        state.chains.get(chain)
            .ok_or_else(|| format!("Chain not found: {:?}", chain))
            .and_then(f)
    })
}

pub fn mutate_chain_state<F, R>(chain: &Chain, f: F) -> Result<R, String>
where F: FnOnce(&mut ChainState) -> Result<R, String> {
    mutate_state(|state| {
        state.chains.get_mut(chain)
            .ok_or_else(|| format!("Chain not found: {:?}", chain))
            .and_then(f)
    })
}
