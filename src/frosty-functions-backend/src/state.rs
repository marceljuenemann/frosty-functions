use std::{cell::RefCell, collections::HashMap};

use alloy::signers::icp::IcpSigner;

use crate::{chain::{Chain}, signer::{IcpSignerId, main_signer}};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::new(None);
}

pub struct State {
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
