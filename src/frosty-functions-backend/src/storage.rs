use candid::{CandidType, Decode, Encode, Nat};
use ic_stable_structures::log::WriteError;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, Log, StableBTreeMap, Storable};
use serde::Deserialize;
use std::borrow::Cow;
use std::cell::RefCell;

use crate::chain::Chain;
use crate::job::{Commit, Job, JobRequest, JobStatus, LogEntry};
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

    // Storage for commits (executions logs).
    static COMMITS: RefCell<Log<Commit, Memory, Memory>> = RefCell::new(
        Log::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
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
        let key = (&request).into();
        if jobs.contains_key(&key) {
            false
        } else {
            jobs.insert(key, Job::new(request));
            true
        }
    })
}

pub fn get_job(chain: &Chain, job_id: Nat) -> Option<Job> {
    let key = JobKey {
        chain: chain.clone(),
        on_chain_id: job_id,
    };
    JOBS.with(|p| p.borrow_mut().get(&key))
}

pub fn update_job<R>(job: &JobRequest, f: impl FnOnce(&mut Job) -> R) -> R {
    JOBS.with(move |p| {
        let key: JobKey = job.into();
        let mut job = p.borrow_mut().get(&key).clone().expect("Job not found");
        let result = f(&mut job);
        p.borrow_mut().insert(key, job);
        result
    })
}

pub fn update_job_status(job: &JobRequest, status: JobStatus) {
    update_job(job, |job| {
        ic_cdk::println!("Updating job status to {:?} for job {:?}", status, job.request.on_chain_id);
        job.status = status;
    })
}

pub fn store_commit(job: &JobRequest, commit: &Commit) -> Result<u64, WriteError> {
    let commit_id = COMMITS.with(|p| {
        p.borrow_mut().append(commit)
    })?;
    update_job(job, |job| job.commit_ids.push(commit_id));
    Ok(commit_id)
}

/// Cross-chain Job ID.
#[derive(Debug, Deserialize, Clone, CandidType, Ord, PartialOrd, PartialEq, Eq)]
struct JobKey {
    pub chain: Chain,
    pub on_chain_id: Nat,
}

impl Into<JobKey> for &JobRequest {
    fn into(self) -> JobKey {
        JobKey {
            chain: self.chain.clone(),
            on_chain_id: self.on_chain_id.clone().map(|nat| nat.into())
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
impl_storable!(Commit);
impl_storable!(LogEntry);
