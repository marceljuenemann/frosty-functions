use std::cell::RefCell;
use std::rc::Rc;

use alloy::signers::icp::IcpSigner;
use candid::CandidType;
use futures::stream::FuturesUnordered;
use futures::StreamExt;

use crate::runtime::{Commit, Job, JobRequest, RuntimeEnvironment};
use crate::runtime::{Execution};

#[derive(Clone, Debug, CandidType)]
pub struct SimulationResult {
    pub job: Job,
    pub commits: Vec<Commit>,
    pub error: Option<String>,
}

pub fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<SimulationResult, String> {
    let env = Rc::new(RefCell::new(SimulationResult {
        job: Job::new(request),
        commits: Vec::new(),
        error: None,
    }));
    let mut execution = Execution::run_main(wasm, env.clone())?;

    // spawn_017_compat executes the future until the first actual cansiter call. Since we
    // shouldn't actually have any cansiter calls during simulation, the following
    // block should execute synchronously.
    let result_local: Rc<RefCell<Option<Result<(), String>>>> = Rc::new(RefCell::new(None));
    let result_async = result_local.clone();
    ic_cdk::futures::spawn_017_compat(async move {
        let result = event_loop(&mut execution).await;
        result_async.borrow_mut().replace(result);
    });

    #[allow(unused_must_use)]
    result_local.borrow().clone().expect("Simulation did not complete synchronously");
    let result = env.borrow().clone();
    Ok(result)
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

impl RuntimeEnvironment for Rc<RefCell<SimulationResult>> {
    fn is_simulation(&self) -> bool {
        true
    }

    fn job_request(&self) -> JobRequest {
        self.borrow().job.request.clone()
    }

    fn charge_fee(&mut self, fee: u64) -> Result<(), String> {
        // TODO: Make gas balance configurable and check against it.
        self.borrow_mut().job.execution_fees += fee;
        Ok(())
    }

    fn charge_gas(&mut self, gas: u64) -> Result<(), String> {
        // TODO: Make gas balance configurable and check against it.
        self.borrow_mut().job.gas_fees += gas;
        Ok(())
    }

    fn commit(&mut self, commit: Commit) {
        self.borrow_mut().commits.push(commit);
    }

    fn caller_wallet(&self) -> Option<IcpSigner> { 
        // TODO: This is very ugly, refactor
        None
    }
}
