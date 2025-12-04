use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

use crate::repository::{FunctionId, FunctionState};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Storage for function definitions and binaries.
    static FUNCTIONS: RefCell<StableBTreeMap<FunctionId, FunctionState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

pub fn store_function(id: FunctionId, state: FunctionState) -> Option<FunctionState> {
    FUNCTIONS.with(|p| p.borrow_mut().insert(id, state))
}

pub fn get_function(id: FunctionId) -> Option<FunctionState> {
    FUNCTIONS.with(|p| p.borrow_mut().get(&id))
}

// TODO: Could turn this into general-purpose proc macro.
impl Storable for FunctionState {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
