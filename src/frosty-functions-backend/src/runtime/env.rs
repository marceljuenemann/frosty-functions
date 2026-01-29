use crate::runtime::{Commit, JobRequest};

/// Trait to be implemented by consumers of the runtime module to provide
/// any functionlity that requires access to the outside world or information.
pub trait RuntimeEnvironment {
    fn is_simulation(&self) -> bool;

    /// Returns the job request that triggered the current execution.
    // TODO: Probably move this out of the runtime and replace with env variables.
    fn job_request(&self) -> JobRequest;

    /// Charges the given fee in the calling currency using the gas balance.
    /// Returns an Error if insufficient funds are available.
    fn charge_fee(&mut self, fee: u64) -> Result<(), String>;

    /// Charges the given fee in the calling currency using the gas balance.
    /// Gas fees will be accounted separately from executions fees as they
    /// are charged by the calling chain rather than by ICP / Frosty.
    /// Returns an Error if insufficient funds are available.
    // TODO: Remove this, no longer allowing native transactions on the main account.
    fn charge_gas(&mut self, gas: u64) -> Result<(), String>;

    /// Submits a commit to be stored persistently.
    fn commit(&mut self, commit: Commit);
}
