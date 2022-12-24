//! 


#![no_std]
#![deny(warnings)]
#![allow(unused)]

#[cfg(feature = "user")]
#[macro_use]
extern crate bitflags;


// mod syscall_asm;
#[macro_use]
mod syscall;

#[cfg(feature = "user")]
mod user;

#[cfg(feature = "user")]
pub use user::*;
pub use syscall::{async_read, ASYNC_SYSCALL_READ, ASYNC_SYSCALL_WRITE};
