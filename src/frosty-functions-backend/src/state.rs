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
