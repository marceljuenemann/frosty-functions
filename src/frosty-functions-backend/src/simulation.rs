use std::cell::RefCell;
use std::rc::Rc;

use alloy::signers::icp::IcpSigner;
use candid::CandidType;

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
    /*
    loop {
        ic_cdk::println!("Awaiting next future");  // TODO: remove
        match execution.next_future().await {
            Some(result) => {
                execution.callback(result)?;
            },
            None => {
                println!("No more futures!");
                break;
            }
        }
    }
    */

    let commits = commits.borrow().clone();
    Ok(SimulationResult {
        commits,
        error: None,
    })
}

struct SimulationEnv {
    job_request: JobRequest,
    commits: Rc<RefCell<Vec<Commit>>>,
}

impl RuntimeEnvironment for SimulationEnv {
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
