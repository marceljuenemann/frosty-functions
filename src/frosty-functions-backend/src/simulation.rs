use crate::runtime::{ExecutionResult, JobRequest};


pub async fn simulate_job(request: JobRequest, wasm: &[u8]) -> Result<ExecutionResult, String> {
    Err("Not implemented yet".to_string())
}
