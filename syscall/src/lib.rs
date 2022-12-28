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
pub use syscall::*;
pub const SYSCALL_READ: usize = 63;


#[cfg(feature = "user")]
mod user;

#[cfg(feature = "user")]
pub use user::*;
