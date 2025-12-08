use std::time::Duration;

use ic_cdk_timers::set_timer;

use crate::runtime::{Commit, JobRequest, JobStatus};
use crate::runtime::{Execution, ExecutionResult};
use crate::signer::signer_for_address;
use crate::storage::{get_function, update_job_status};

const FUEL_PER_BATCH: u64 = 1_000_000;

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

pub async fn execute_job(request: JobRequest, wasm: &[u8]) -> Result<(), String> {
    update_job_status(&request, JobStatus::Executing);
    let signer = signer_for_address(&request.caller).await?;
    // TODO: Maybe pass int ExecutionContext
    let mut execution = Execution::init(request.clone(), wasm, false, signer)?;
    // TODO: Commit after errors.
    execution.call_by_name("main".to_string())?;

    // TODO: Start all async tasks before commiting?
    // TODO: with_commit rather than manual commit calls.
    // TODO: Probably need a on_commit callback for async functions with multiple commits.
    let commit = execution.commit("main()".to_ascii_lowercase())?;
    store_commit(&request, &commit)?;

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

    // TODO: Handled errors
    update_job_status(&request, JobStatus::Completed);
    Ok(())
}

fn store_commit(job: &JobRequest, commit: &Commit) -> Result<u64, String> {
    crate::storage::store_commit(job, commit)
        .map_err(|err| format!("Failed to store commit: {:?}", err))
}

pub async fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<ExecutionResult, String> {
    Err("Not implemented yet".to_string())
}
