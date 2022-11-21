#![no_std]
#![feature(const_for)]
#![feature(const_mut_refs)]

mod coroutine;
mod task_waker;
mod executor;
mod config;
mod bitmap;
pub mod fun_offset;

extern crate alloc;

pub use executor::Executor;
pub use coroutine::{CoroutineId, Coroutine};
pub use config::PRIO_NUM;
pub use config::CBQ_MAX;
pub use config::MAX_PROC_NUM;
use bitmap::BitMap;
// pub use task_waker::TaskWaker;