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
        futures: FuturesUnordered::new(),
    };

    let mut execution = Execution::run_main(wasm, env)?;
    ic_cdk::println!("run_main returned");

    let mut futures = FuturesUnordered::new();
    loop {
        // TODO: Looks like this could just go into Execution::run_loop() or similar. 
        // In the Future it will need to return a continuation token or similar.
        // After a timer, could also just invoke run_loop, which exists immediately if
        // the loop is already running. At the end of the loop we check whether all "threads"
        // finished, or not.

        // Importantly, we should have FuturesUnordered as well as a queue for AsyncResults.
        // Then we can choose not to invoke the callback unless we have enough instructions left.
        if !execution.ctx().async_tasks.is_empty() {
            ic_cdk::println!("Processing async tasks: {}", execution.ctx().async_tasks.len());
            for task in execution.ctx().async_tasks.drain(..) {
                futures.push(task.future);
            }
        }

        ic_cdk::println!("Awaiting next future. Count: {}", futures.len());
        match futures.next().await {
            Some(result) => {
                println!("Finished future");
            },
            None => {
                println!("No more futures!");
                break;
            }
        }
    }

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
    pub futures: FuturesUnordered<Pin<Box<dyn Future<Output = AsyncResult> + 'static>>>,
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

    /*
    fn spawn(&self, future: impl Future<Output = AsyncResult> + 'static) {
        // ic_cdk::futures::spawn_migratory(future);
        self.futures.push(Box::pin(future));
    }
    */

    fn caller_wallet(&self) -> IcpSigner { 
        self.caller_wallet.clone()
    }
}
