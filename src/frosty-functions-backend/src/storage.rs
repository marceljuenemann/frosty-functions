use candid::{CandidType, Decode, Encode, Nat};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

use crate::chain::Chain;
use crate::job::{Job, JobRequest};
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

    // Storage for job requests and status.
    static JOBS: RefCell<StableBTreeMap<JobKey, Job, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );
}

pub fn store_function(id: FunctionId, state: FunctionState) -> Option<FunctionState> {
    FUNCTIONS.with(|p| p.borrow_mut().insert(id, state))
}

pub fn get_function(id: FunctionId) -> Option<FunctionState> {
    FUNCTIONS.with(|p| p.borrow_mut().get(&id))
}

pub fn create_job(request: JobRequest) -> bool {
    JOBS.with(|p| {
        let mut jobs = p.borrow_mut();
        let key = request.clone().into();
        if jobs.contains_key(&key) {
            false
        } else {
            jobs.insert(key, Job::new(request));
            true
        }
    })
}

/// Cross-chain Job ID.
#[derive(Debug, Clone, CandidType, Ord, PartialOrd, PartialEq, Eq)]
struct JobKey {
    pub chain: Chain,
    pub on_chain_id: Nat,
}

impl Into<JobKey> for JobRequest {
    fn into(self) -> JobKey {
        JobKey {
            chain: self.chain,
            on_chain_id: self.on_chain_id.map(|nat| nat.into())
                .expect("Can only store Jobs with on_chain_id set"),
        }
    }
}

// Implement Storable for a CandidType.
macro_rules! impl_storable {
    ($type: ty) => {
        impl Storable for $type {
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
    };
}

impl_storable!(FunctionState);
impl_storable!(JobKey);  // TODO: Might want to use Bound::FixedSize here.
impl_storable!(Job);
