use std::fmt::Debug;

use crate::supervisor::Supervisor;
use crate::Worker;

pub enum Task {
    Worker(Box<dyn Worker>),
    Supervisor(Supervisor),
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::Worker(_worker_task) => {
                write!(f, "Worker")
            }
            Task::Supervisor(s) => s.fmt(f),
        }
    }
}
