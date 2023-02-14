mod mail;
mod pipe;
mod serial;
pub mod stdio;

use crate::mm::UserBuffer;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub use mail::{MailBox, Socket};
pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn write(&self, buf: UserBuffer, is_nonblock: bool) -> Result<usize, isize>;
    fn aread(&self, buf: UserBuffer, tid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;

}

pub use pipe::{make_pipe, Pipe};
pub use serial::Serial;
pub use stdio::{Stdin, Stdout};
