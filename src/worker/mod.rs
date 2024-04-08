use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use self::restartable::Restartable;
use crate::{BackoffPolicy, RestartPolicy};

pub mod backoff;
pub mod backoff_policy;
pub mod restartable;
pub mod watcher;

/// A trait representing a worker that can be restarted.
pub trait Worker: Debug + Send + Restartable {
    /// The initialization entrypoint for the worker. This is called when the
    /// worker is started, and can be called an infinite number of times if
    /// indefinite restarts are permitted. Therefore, it should be safe to call
    /// this repeatedly.
    fn init(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    /// Returns the restart policy for worker.
    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::default()
    }

    /// Returns the backoff policy for the worker.
    fn backoff_policy(&self) -> BackoffPolicy {
        BackoffPolicy::default()
    }
}
