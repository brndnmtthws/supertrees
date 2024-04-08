#![warn(missing_docs)]
//! # Supervision trees for async Rust, inspired by Erlang/OTP
//!
//! This crate provides a supervision tree implementation for async Rust,
//! inspired by Erlang/OTP. It provides a [`Supertree`] struct that can be used
//! to create a supervision tree, and a [`Worker`] trait that can be implemented
//! by workers that can be added to the supervision tree.
//!
//! This crate is designed to be used with async Rust and the [Tokio](https://tokio.rs/) runtime, but
//! it could theoretically be used with other async runtimes as well.
//!
//! In its current state, this crate is considered experimental and should not
//! be used for production services, unless you are very excited about the idea
//! and would be willing to contribute to the development of the crate. Notably,
//! this crate lacks a lot of the features that are present in Erlang/OTP, such
//! as monitoring, tracing, and distributed messaging, although Tokio provides a
//! tracing and metrics system that could be used in conjunction with this crate
//! (it has just not been tested yet).
//!
//! ## Example
//!
//! A basic example of using this crate to create a supervision tree with a root
//! supervisor, two sub-supervisors, and three workers:
//!
//! ```rust
//! use supertrees::{Restartable, Supertree, Worker};
//!
//! #[derive(Debug)]
//! struct MyWorker {
//!     num: i32,
//! }
//!
//! impl MyWorker {
//!     fn new(num: i32) -> Self {
//!         Self { num }
//!     }
//! }
//!
//! impl Worker for MyWorker {
//!     // init() is called before the worker is started, and will be called
//!     // after each subsequent restart, so it should be safe to call this
//!     // repeatedly and  any state that needs to be reset should be reset here.
//!     fn init(
//!         &self,
//!     ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
//!         let num = self.num;
//!         Box::pin(async move {
//!             println!("hi, I'm worker num={num} :)");
//!         })
//!     }
//! }
//!
//! // We must provide a restart and backoff policy for the worker, but we can
//! // use the default policies.
//! impl Restartable for MyWorker {}
//!
//! let root = Supertree::new()
//!     .add_worker(MyWorker::new(1))
//!     .add_supervisor(|s| {
//!         s.add_worker(MyWorker::new(2))
//!             .add_supervisor(|s| s.add_worker(MyWorker::new(3)))
//!     });
//! dbg!(&root);
//!
//! // Now you can start the supervision tree, which will run forever.
//! // Uncomment the line below to run the supervision tree.
//! // root.start();
//! ```

pub use supervisor::Supervisor;
pub use worker::backoff_policy::BackoffPolicy;
pub use worker::restartable::{RestartPolicy, Restartable};
pub use worker::Worker;

mod fork;
mod process;
mod supervisor;
mod syscall;
mod task;
mod worker;

#[derive(Debug)]
/// Represents a supertree, which is a supervision tree that contains a root
/// supervisor.
pub struct Supertree {
    root: Supervisor,
}

/// Represents a Supervision tree, which is a hierarchical structure used for
/// managing workers and supervisors.
///
/// The Supertree struct provides methods for creating a new Supertree,
/// configuring backoff and restart policies, running the Supertree, adding
/// workers and supervisors to the Supertree.
impl Supertree {
    /// Creates a new Supertree with a default root supervisor.
    pub fn new() -> Self {
        Self {
            root: Supervisor::new(unsafe { libc::getpid() }),
        }
    }

    /// Sets the backoff policy for the Supertree.
    pub fn with_backoff_policy(mut self, backoff_policy: BackoffPolicy) -> Self {
        self.root = self.root.with_backoff_policy(backoff_policy);
        self
    }

    /// Sets the restart policy for the Supertree.
    pub fn with_restart_policy(mut self, restart_policy: RestartPolicy) -> Self {
        self.root = self.root.with_restart_policy(restart_policy);
        self
    }

    /// Starts the supervision tree, starting the root supervisor and all its
    /// workers and supervisors.
    pub fn start(mut self) {
        self.root.run();
    }

    /// Adds a worker to the Supertree and returns a new Supertree with the
    /// added worker.
    pub fn add_worker(mut self, worker: impl Worker + 'static) -> Self {
        self.root = self.root.add_worker(worker);
        self
    }

    /// Adds a supervisor to the Supertree and returns a new Supertree with the
    /// added supervisor. The supervisor is created by applying the given
    /// closure to the current root supervisor.
    pub fn add_supervisor<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Supervisor) -> Supervisor,
    {
        self.root = self.root.add_supervisor(f);
        self
    }
}

impl Default for Supertree {
    fn default() -> Self {
        Self::new()
    }
}
