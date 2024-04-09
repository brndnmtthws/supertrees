pub mod process_group;

use std::fmt::Debug;

use crate::worker::restartable::Restartable;

pub trait Process: Restartable + Debug {
    fn start(&mut self);
}
