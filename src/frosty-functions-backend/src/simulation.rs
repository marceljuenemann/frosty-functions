use std::cell::RefCell;
use std::rc::Rc;

use alloy::signers::icp::IcpSigner;
use candid::CandidType;
use futures::stream::FuturesUnordered;
use futures::StreamExt;

use crate::runtime::{Commit, JobRequest, RuntimeEnvironment};
use crate::runtime::{Execution};

#[derive(Clone, Debug, CandidType)]
pub struct SimulationResult {
    pub commits: Vec<Commit>,
    pub error: Option<String>,
}

pub fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<SimulationResult, String> {
    let commits = Rc::new(RefCell::new(Vec::new()));
    let env = SimulationEnv {
        job_request: request.clone(),
        commits: commits.clone(),
    };
    let mut execution = Execution::run_main(wasm, env)?;

    // spawn_017_compat executes the future until the first actual cansiter call. Since we
    // shouldn't actually have any cansiter calls during simulation, the following
    // block should execute synchronously.
    let result_local: Rc<RefCell<Option<Result<(), String>>>> = Rc::new(RefCell::new(None));
    let result_async = result_local.clone();
    ic_cdk::futures::spawn_017_compat(async move {
        let result = event_loop(&mut execution).await;
        result_async.borrow_mut().replace(result);
    });

    let result = result_local.borrow().clone().expect("Simulation did not complete synchronously");
    let commits = commits.borrow().clone();
    Ok(SimulationResult {
        commits,
        error: result.err(),
    })
}

async fn event_loop(execution: &mut Execution) -> Result<(), String> {
    let mut futures = FuturesUnordered::new();
    loop {
        while let Some(async_future) = execution.next_queued_future() {
            futures.push(async_future);
        }

        let async_result = futures.next().await;
        match async_result {
            Some(result) => {
                let callback_result = execution.callback(result);
                if callback_result.is_err() {
                    return Err(format!("Error during simulation: {}", callback_result.unwrap_err()));
                }
            },
            None => {
                return Ok(());
            }
        }
    }
}

struct SimulationEnv {
    job_request: JobRequest,
    commits: Rc<RefCell<Vec<Commit>>>,
}

impl RuntimeEnvironment for SimulationEnv {
    fn is_simulation(&self) -> bool {
        true
    }

    fn job_request(&self) -> &JobRequest {
        &self.job_request
    }
    
    fn commit(&self, commit: Commit) {
        self.commits.borrow_mut().push(commit);
    }

    fn caller_wallet(&self) -> IcpSigner { 
        todo!("[SIMULATION] caller_wallet")
    }
}
