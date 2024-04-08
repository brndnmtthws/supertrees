use super::backoff_policy::BackoffPolicy;

/// Represents the restart policy for a process or worker task.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum RestartPolicy {
    /// Always restart the process or task.
    #[default]
    Always,
    /// Restart the process or task once.
    Once,
    /// Never restart the process or task.
    Never,
}

/// Trait for restartable processes or worker tasks.
pub trait Restartable {
    /// Returns the restart policy for the process or task.
    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::default()
    }

    /// Returns the backoff policy for the process or task.
    fn backoff_policy(&self) -> BackoffPolicy {
        BackoffPolicy::default()
    }
}
