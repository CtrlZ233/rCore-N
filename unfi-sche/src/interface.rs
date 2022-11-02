
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use core::pin::Pin;
use core::future::Future;
use core::task::{Poll, Context};
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::executor::EXECUTOR;
use crate::thread::Thread;
use runtime::Coroutine;
use spin::Mutex;
use crate::syscall::*;

#[no_mangle]
#[inline(never)]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    let task = Arc::new(Coroutine::spawn(Mutex::new(future), prio));
    unsafe{ EXECUTOR.as_mut().unwrap().add_coroutine(task); }
    println!("add task ok");
}

#[no_mangle]
pub fn run(){
    let ex = unsafe { EXECUTOR.as_mut().unwrap() };
    if ex.is_empty() { sys_exit(0); }
    let mut thread = Thread::new();
    thread.execute();
}

#[no_mangle]
pub fn poll_future() {
    let ex = unsafe { EXECUTOR.as_mut().unwrap() };
    let mut wait_task = BTreeSet::new();
    loop {
        let task = ex.fetch();
        if task.is_none() {
            if wait_task.is_empty() {
                break;
            }
            continue;
        }
        println!("run coroutine {}", task.clone().unwrap().cid.0);
        let waker = ex.get_waker(task.clone().unwrap().cid, task.clone().unwrap().prio);

        // creat Context
        let mut context = Context::from_waker(&*waker);
        match task.clone().unwrap().future.lock().as_mut().poll(&mut context) {
            Poll::Pending => {
                wait_task.insert(task.unwrap().cid.0);
            }
            Poll::Ready(()) => {
                // remove task
                wait_task.remove(&task.clone().unwrap().cid.0);
                ex.del_task(task.clone().unwrap().cid);

            }
        };
        // println!("yield thread");
        // yield_thread();
    }
}

pub fn wake_coroutine(cid: usize) {
    let ex = unsafe { EXECUTOR.as_mut().unwrap() };
    ex.block_task(cid);
}

pub fn get_current_coroutine_id() -> usize {
    let ex = unsafe { EXECUTOR.as_mut().unwrap() };
    ex.get_current_id().0
}

pub fn yield_thread() {
    let sp_ptr = 1usize << 38;
    unsafe {
        core::arch::asm!(
            "mv sp, {sp_ptr}",
            "ld ra, {run}",
            "ret",
            sp_ptr = in(reg) sp_ptr,
            run = sym run,
            options(noreturn)
        )
    }
}