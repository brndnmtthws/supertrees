use std::collections::HashMap;
use std::{io, thread};

use libc::pid_t;
use log::debug;

use super::Process;
use crate::fork::{fork, ForkResult};
use crate::syscall::syscall;
use crate::worker::backoff::{Backoff, BackoffResult};

pub struct ProcessGroup {
    processes: Vec<Box<dyn Process>>,
}

impl ProcessGroup {
    pub fn new() -> Self {
        Self { processes: vec![] }
    }

    pub fn add_process(&mut self, process: Box<dyn Process>) {
        self.processes.push(process);
    }

    fn fork(process: &mut Box<dyn Process>) -> io::Result<pid_t> {
        debug!("forking new child process");
        let fork_result = fork()?;

        match fork_result {
            ForkResult::Child => {
                process.start();
                Ok(0)
            }
            ForkResult::Parent(child_pid) => {
                Self::handle_child(child_pid)?;
                Ok(child_pid)
            }
        }
    }

    fn handle_child(child_pid: pid_t) -> io::Result<()> {
        debug!("child pid={child_pid} started");
        unsafe {
            // set process group ID for current process
            syscall(libc::setpgid(0, 0))?;
            // set process group ID for child process
            syscall(libc::setpgid(child_pid, libc::getpid()))?;
        };
        Ok(())
    }

    fn send_sigterm(child_pid: pid_t) {
        debug!("sending SIGTERM to {child_pid}");
        unsafe {
            libc::kill(child_pid, libc::SIGTERM);
        }
    }

    fn terminate<P>(processes: &mut HashMap<pid_t, P>) {
        debug!("terminating remaining children");
        processes
            .iter()
            .filter_map(|(child_pid, _)| match child_pid {
                0 => None,
                _ => Some(child_pid),
            })
            .for_each(|child_pid| Self::send_sigterm(*child_pid));
        processes.clear();
    }

    pub fn run(self) {
        let count = self.processes.len();
        debug!("starting process group with {count} processes");

        let mut processes: HashMap<pid_t, Backoff<dyn Process>> = HashMap::new();

        for mut process in self.processes.into_iter() {
            let child_pid = Self::fork(&mut process).expect("fork failed");
            if child_pid != 0 {
                processes.insert(child_pid, Backoff::new(process));
            } else {
                // pid 0 are forked children returning/exiting, as such we can ignore
                // them and return early.
                return;
            }
        }

        let pid = unsafe { libc::getpid() };
        while !processes.is_empty() {
            let mut status: libc::c_int = 0;
            debug!("waiting on children from pid={pid}");
            match unsafe { syscall(libc::waitpid(-pid, &mut status, 0)) } {
                Ok(ret) => {
                    debug!("waitpid returned ret={ret} status={status}");
                    let exited = libc::WIFEXITED(status);
                    let exit_status = libc::WEXITSTATUS(status);
                    let signaled = libc::WIFSIGNALED(status);
                    if signaled {
                        let signal = libc::WTERMSIG(status);
                        debug!("waitpid interrupted by signal={signal}");
                        Self::terminate(&mut processes);
                    } else if exited {
                        debug!("child pid={ret} exited with exit_status={exit_status}");
                        Self::send_sigterm(ret);
                        match processes.remove(&ret) {
                            Some(mut process) => {
                                if let BackoffResult::RetryAfterDelay(delay) = process.maybe_delay()
                                {
                                    debug!("retrying child pid={ret} after delay={delay:?}");
                                    thread::sleep(delay);
                                    let child_pid = Self::fork(&mut process).expect("fork failed");
                                    processes.insert(child_pid, process);
                                }
                            }
                            None => {
                                debug!("pid={ret} not in process map, this shouldn't happen")
                            }
                        }
                    }
                }
                Err(err) => {
                    debug!("waitpid err={err}, stopping process group");
                    Self::terminate(&mut processes);
                    break;
                }
            }
        }
    }
}
