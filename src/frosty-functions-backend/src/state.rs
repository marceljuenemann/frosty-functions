use std::{cell::RefCell, collections::HashMap};

use crate::chain::ChainState;

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        chains: HashMap::new(),
    });
}

pub struct State {
    pub chains: HashMap<String, ChainState>,
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with_borrow(|s| f(s))
}

pub fn mutate_state<F, R>(f: F) -> R where F: FnOnce(&mut State) -> R {
    STATE.with_borrow_mut(|s| f(s))
}

pub fn read_chain_state<R>(chain_id: &String, f: impl FnOnce(&ChainState) -> Result<R, String>) -> Result<R, String> {
    read_state(|state| {
        state.chains.get(chain_id)
            .ok_or_else(|| format!("Chain not found: {}", chain_id))
            .and_then(f)
    })
}

pub fn mutate_chain_state<F, R>(chain_id: &String, f: F) -> Result<R, String>
where F: FnOnce(&mut ChainState) -> Result<R, String> {
    mutate_state(|state| {
        state.chains.get_mut(chain_id)
            .ok_or_else(|| format!("Chain not found: {}", chain_id))
            .and_then(f)
    })
}
