use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

use super::restartable::{RestartPolicy, Restartable};

#[derive(Debug)]
pub struct Backoff<Inner: ?Sized> {
    inner: Box<Inner>,
    last_action: Option<Instant>,
}

pub enum BackoffResult {
    RetryAfterDelay(Duration),
    GiveUp,
}

impl<Inner: ?Sized> Backoff<Inner> {
    pub fn new(inner: Box<Inner>) -> Self {
        Self {
            inner,
            last_action: None,
        }
    }

    pub fn into_inner(self) -> Box<Inner> {
        self.inner
    }
}

impl<Inner: Restartable + ?Sized> Backoff<Inner> {
    pub fn maybe_delay(&mut self) -> BackoffResult {
        if self.inner.restart_policy() == RestartPolicy::Never {
            return BackoffResult::GiveUp;
        }
        let backoff_policy = self.inner.backoff_policy();
        let now = Instant::now();
        let delay = if let Some(ts) = self.last_action {
            let diff = now - ts;
            if diff < backoff_policy.max_delay() {
                let delay = Duration::from_millis(
                    (diff.as_millis() as f64 * backoff_policy.multiplier()) as u64,
                );
                if delay > backoff_policy.max_delay() {
                    backoff_policy.max_delay()
                } else if delay < backoff_policy.min_delay() {
                    backoff_policy.min_delay()
                } else {
                    delay
                }
            } else if diff > backoff_policy.reset_after() {
                backoff_policy.min_delay()
            } else {
                backoff_policy.max_delay()
            }
        } else {
            backoff_policy.min_delay()
        };
        let ret = match self.inner.restart_policy() {
            RestartPolicy::Always => BackoffResult::RetryAfterDelay(delay),
            RestartPolicy::Once if self.last_action.is_none() => {
                BackoffResult::RetryAfterDelay(delay)
            }
            _ => BackoffResult::GiveUp,
        };
        self.last_action = Some(now);
        ret
    }
}

impl<Inner: ?Sized> Deref for Backoff<Inner> {
    type Target = Box<Inner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Inner: ?Sized> DerefMut for Backoff<Inner> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
