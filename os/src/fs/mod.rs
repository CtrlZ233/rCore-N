mod mail;
mod pipe;
mod serial;
pub mod stdio;

use crate::mm::UserBuffer;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin, task::{Poll, Context}};

pub use mail::{MailBox, Socket};
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn write(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn awrite(&self, buf: UserBuffer, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;
    fn aread(&self, buf: UserBuffer, cid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;

}

pub use pipe::{make_pipe, Pipe};
pub use serial::Serial;
pub use stdio::{Stdin, Stdout};


pub struct ReadHelper(usize);

impl ReadHelper {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Future for ReadHelper {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0 += 1;
        if (self.0 & 1) == 1 {
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}