use alloy::signers::icp::IcpSigner;

use crate::runtime::{Commit, JobRequest};

/// Trait to be implemented by consumers of the runtime module to provide
/// any functionlity that requires access to the outside world or information.
pub trait RuntimeEnvironment {
    fn is_simulation(&self) -> bool;

    /// Returns the job request that triggered the current execution.
    fn job_request(&self) -> &JobRequest;

    /// Submits a commit to be stored persistently.
    fn commit(&self, commit: Commit);

    /// Returns the shared wallet for the caller of the execution.
    // TODO: Refactor this to make more sense in simulations. Maybe just split into
    // separate calls. Should be possible to borrow execution context from async context
    fn caller_wallet(&self) -> Option<IcpSigner>;
}
