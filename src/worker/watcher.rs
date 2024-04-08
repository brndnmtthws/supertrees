use std::ops::Deref;

use log::debug;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

use crate::process::Process;
use crate::worker::backoff::{Backoff, BackoffResult};
use crate::worker::Restartable;
use crate::Worker;

#[derive(Debug)]
pub struct Watcher {
    workers: Vec<Box<dyn Worker>>,
}

impl Watcher {
    pub fn new(workers: Vec<Box<dyn Worker>>) -> Self {
        Self { workers }
    }

    fn start_worker(joinset: &mut JoinSet<Box<dyn Worker>>, worker: Box<dyn Worker>) {
        debug!("starting worker={worker:?}");
        joinset.spawn(async move {
            let mut backoff = Backoff::new(worker);
            loop {
                let f = backoff.init();
                f.await;
                if let BackoffResult::RetryAfterDelay(delay) = backoff.maybe_delay() {
                    debug!(
                        "worker stopped, retrying after delay={delay:?} for worker={:?}",
                        backoff.deref()
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    break;
                }
            }
            backoff.into_inner()
        });
    }

    fn start_workers(&mut self) -> JoinSet<Box<dyn Worker>> {
        let mut joinset = JoinSet::new();
        let workers = std::mem::take(&mut self.workers);
        for worker in workers.into_iter() {
            Self::start_worker(&mut joinset, worker);
        }
        joinset
    }

    fn start(&mut self) {
        debug!("starting tokio runtime");
        let rt = Runtime::new().expect("failed to start runtime");
        rt.block_on(async move {
            let mut joinset = self.start_workers();
            while let Some(Ok(worker)) = joinset.join_next().await {
                debug!("joined worker={:?}", worker);
            }
        });
    }
}

impl Process for Watcher {
    fn start(&mut self) {
        self.start();
    }
}

impl Restartable for Watcher {}
