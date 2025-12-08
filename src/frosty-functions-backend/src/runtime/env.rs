use std::future::Future;

use alloy::signers::icp::IcpSigner;

use crate::runtime::{AsyncResult, Commit, JobRequest};

/// Trait to be implemented by consumers of the runtime module to provide
/// any functionlity that requires access to the outside world or information.
pub trait RuntimeEnvironment {

    /// Returns the job request that triggered the current execution.
    fn job_request(&self) -> &JobRequest;

    /// Submits a commit to be stored persistently.
    fn commit(&self, commit: Commit);

    /// Spawns an async task to be executed. The AsyncResult should be passed into
    /// Execution::callback when complete. Note that all tasks can be executed
    /// without actually triggering an asynchronous inter-canister call, as long as
    /// the implementations of this trait don't actually perform any.
    //fn spawn(&self, future: impl Future<Output = AsyncResult> + 'static);

    /// Returns the shared wallet for the caller of the execution.
    fn caller_wallet(&self) -> IcpSigner;
}
