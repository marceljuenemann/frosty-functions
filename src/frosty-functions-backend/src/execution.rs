use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use alloy::signers::icp::IcpSigner;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use ic_cdk_timers::set_timer;

use crate::runtime::{AsyncResult, Commit, JobRequest, JobStatus, RuntimeEnvironment};
use crate::runtime::{Execution};
use crate::signer::signer_for_address;
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
        let wasm = function.unwrap().definition.binary;
        let result = execute_job(job_request, &wasm).await;
        // TODO: Handle errors properly here. Ideally change to void return type.
        if result.is_err() {
            ic_cdk::println!("Job execution failed: {}", result.as_ref().unwrap_err());
        }
    });
}

// TODO: Better error handling.
pub async fn execute_job(request: JobRequest, wasm: &[u8]) -> Result<(), String> {
    update_job_status(&request, JobStatus::Executing);
    let env = ExecutionEnvironment {
        job_request: request.clone(),
        caller_wallet: signer_for_address(&request.caller).await?,
    };

    // TODO: Enable long running tasks in main().
    let mut execution = Execution::run_main(wasm, env)?;
    ic_cdk::println!("run_main returned");  // TODO: remove

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

    // TODO: Handled errors
    update_job_status(&request, JobStatus::Completed);
    Ok(())
}

struct ExecutionEnvironment {
    job_request: JobRequest,
    caller_wallet: IcpSigner,
}

impl RuntimeEnvironment for ExecutionEnvironment {
    fn job_request(&self) -> &JobRequest {
        &self.job_request
    }
    
    fn commit(&self, commit: Commit) {
        crate::storage::store_commit(&self.job_request, &commit)
            .expect("Failed to store commit");
    }

    fn caller_wallet(&self) -> IcpSigner { 
        self.caller_wallet.clone()
    }
}
