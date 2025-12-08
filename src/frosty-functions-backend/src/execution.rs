use std::time::Duration;

use alloy::signers::icp::IcpSigner;
use ic_cdk_timers::set_timer;

use crate::runtime::{Commit, JobRequest, JobStatus, RuntimeEnvironment};
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

    let execution = Execution::run_main(wasm, env)?;

    // TODO: Spawn and handle all async tasks now
    /*
    while !execution.store.data().async_tasks.is_empty() {
        ic_cdk::println!("Processing {} async tasks...", execution.store.data().async_tasks.len());

        // TODO: Wait for multiple tasks in parallel using spawn.
        let task = execution.store.data_mut().async_tasks.remove(0);
        let result = task.future.await;
        execution.callback(task.id, &result)?;
        // TODO: Start more tasks.
        // TODO: Set source
        let commit = execution.commit(format!("Task #{}: {}", task.id, task.description))?;
        store_commit(&request, &commit)?;
    }
    */

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
