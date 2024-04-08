use std::fmt::{Debug, Display};

use libc::pid_t;

use crate::process::process_group::ProcessGroup;
use crate::process::Process;
use crate::task::Task;
use crate::worker::backoff_policy::BackoffPolicy;
use crate::worker::restartable::{RestartPolicy, Restartable};
use crate::worker::watcher::Watcher;
use crate::worker::Worker;

/// Represents a supervisor that manages a collection of supervisors and tasks.
pub struct Supervisor {
    root_pid: pid_t,
    tasks: Vec<Task>,
    backoff_policy: BackoffPolicy,
    restart_policy: RestartPolicy,
}

impl Debug for Supervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Supervisor")
            .field("root_pid", &self.root_pid)
            .field("tasks", &self.tasks)
            .finish()
    }
}

impl Supervisor {
    pub(crate) fn new(root_pid: pid_t) -> Self {
        Self {
            root_pid,
            tasks: vec![],
            backoff_policy: BackoffPolicy::default(),
            restart_policy: RestartPolicy::default(),
        }
    }

    /// Sets the backoff policy for the Supervisor.
    pub fn with_backoff_policy(mut self, backoff_policy: BackoffPolicy) -> Self {
        self.backoff_policy = backoff_policy;
        self
    }

    /// Sets the restart policy for the Supervisor.
    pub fn with_restart_policy(mut self, restart_policy: RestartPolicy) -> Self {
        self.restart_policy = restart_policy;
        self
    }

    pub(crate) fn run(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);
        let (workers, supervisors): (Vec<_>, Vec<_>) = tasks
            .into_iter()
            .partition(|w| matches!(w, Task::Worker(_)));
        let worker_watcher = Watcher::new(
            workers
                .into_iter()
                .filter_map(|w| match w {
                    Task::Worker(w) => Some(w),
                    _ => None,
                })
                .collect(),
        );

        let mut pg = ProcessGroup::new();
        pg.add_process(Box::new(worker_watcher));
        for supervisor in supervisors.into_iter() {
            if let Task::Supervisor(s) = supervisor {
                pg.add_process(Box::new(s))
            }
        }

        pg.run();
    }

    /// Adds a worker to the supervisor.
    pub fn add_worker(mut self, worker: impl Worker + 'static) -> Self {
        let _id = self.tasks.len();
        self.tasks.push(Task::Worker(Box::new(worker)));
        self
    }

    /// Adds a new child supervisor to the current one, calling the closure
    /// provided with the new supervisor.
    pub fn add_supervisor<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        self.tasks
            .push(Task::Supervisor(f(Supervisor::new(self.root_pid))));
        self
    }
}

impl Process for Supervisor {
    fn start(&mut self) {
        self.run();
    }
}

impl Display for Supervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Restartable for Supervisor {
    fn backoff_policy(&self) -> BackoffPolicy {
        self.backoff_policy
    }

    fn restart_policy(&self) -> RestartPolicy {
        self.restart_policy
    }
}
