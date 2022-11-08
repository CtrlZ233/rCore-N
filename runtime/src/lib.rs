#![no_std]
#![feature(const_for)]
#![feature(const_mut_refs)]

mod coroutine;
mod task_waker;
mod executor;
mod config;

extern crate alloc;

pub use executor::Executor;
pub use coroutine::{CoroutineId, Coroutine};
pub use config::PRIO_NUM;
pub use task_waker::TaskWaker;