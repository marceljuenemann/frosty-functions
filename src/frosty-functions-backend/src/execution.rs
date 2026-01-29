use std::time::Duration;

use alloy::signers::icp::IcpSigner;
use candid::Nat;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use ic_cdk_timers::set_timer;

use crate::runtime::{Commit, JobRequest, JobStatus, RuntimeEnvironment};
use crate::runtime::{Execution};
use crate::storage::{get_function, update_job_status};

pub fn schedule_job(job_request: &JobRequest) {
    let function = get_function(job_request.function_hash.to_vec());
    if function.is_none() {
        update_job_status(&job_request, JobStatus::Failed("No WASM binary found for function".to_string()));
        return;
    }
    
    // Schedule execution of the job in a new IC message in case it panics.
    // TODO: Don't schedule more than X jobs at once.
    let job_request = job_request.clone();
    let timer_id = set_timer(Duration::from_secs(0), async move {
        update_job_status(&job_request, JobStatus::Executing);
        let wasm = function.unwrap().definition.binary;
        let result =  match execute_job(&job_request, &wasm).await {
            Ok(_) => JobStatus::Completed,
            Err(err) => JobStatus::Failed(err),
        };
        update_job_status(&job_request, result);
    });
}

// TODO: Better error handling.
async fn execute_job(request: &JobRequest, wasm: &[u8]) -> Result<(), String> {
    let env = ExecutionEnvironment {
        job_request: request.clone()
    };

    // TODO: Enable long running tasks in main().
    let mut execution = Execution::run_main(wasm, env)?;

    let mut futures = FuturesUnordered::new();
    loop {
        // Move Futures from queue to FuturesUnordered
        // TODO: Turn this into a one-liner
        while let Some(async_future) = execution.next_queued_future() {
            futures.push(async_future);
        }

        match futures.next().await {
            Some(result) => {
                execution.callback(result)?;
            },
            None => {
                break;
            }
        }
    }
    Ok(())
}

struct ExecutionEnvironment {
    job_request: JobRequest
}

impl RuntimeEnvironment for ExecutionEnvironment {
    fn is_simulation(&self) -> bool {
        false
    }

    fn job_request(&self) -> JobRequest {
        self.job_request.clone()
    }

    fn charge_fee(&mut self, fee: u64) -> Result<(), String> {
        crate::storage::update_job(&self.job_request, |job| {
            let remaining = job.remaining_gas();
            if Nat::from(fee) > remaining {
                return Err(format!("Insufficient gas. Tried to charge {}, but only {} remaining", fee, remaining));
            }
            job.execution_fees += fee;
            Ok(())
        })
    }

    fn charge_gas(&mut self, gas: u64) -> Result<(), String> {
        crate::storage::update_job(&self.job_request, |job| {
            let remaining = job.remaining_gas();
            if Nat::from(gas) > remaining {
                return Err(format!("Insufficient gas. Tried to charge {}, but only {} remaining", gas, remaining));
            }
            job.gas_fees += gas;
            Ok(())
        })
    }

    fn commit(&mut self, commit: Commit) {
        crate::storage::store_commit(&self.job_request, &commit)
            .expect("Failed to store commit");
    }
}
