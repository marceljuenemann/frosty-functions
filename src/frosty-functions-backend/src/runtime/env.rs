use alloy::signers::icp::IcpSigner;

use crate::runtime::{Commit, JobRequest};

/// Trait to be implemented by consumers of the runtime module to provide
/// any functionlity that requires access to the outside world or information.
pub trait RuntimeEnvironment {

    /// Returns the job request that triggered the current execution.
    fn job_request(&self) -> &JobRequest;

    /// Submits a commit to be stored persistently.
    // TODO: Introduce custom error type?
    fn commit(&self, commit: Commit);

    /// Returns the shared wallet for the caller of the execution.
    fn caller_wallet(&self) -> IcpSigner;
}
