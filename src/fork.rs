use std::io;

use libc::pid_t;

use crate::syscall::syscall;

#[derive(Debug)]
pub enum ForkResult {
    Parent(pid_t),
    Child,
}

impl ForkResult {
    fn new(pid: pid_t) -> io::Result<Self> {
        match pid {
            0 => Ok(Self::Child),
            pid => Ok(Self::Parent(pid)),
        }
    }
}

pub fn fork() -> io::Result<ForkResult> {
    unsafe { syscall(libc::fork()) }.map(ForkResult::new)?
}
