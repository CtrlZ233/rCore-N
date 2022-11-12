
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use core::pin::Pin;
use core::future::Future;
use core::task::{Poll, Context, Waker};
use alloc::sync::Arc;
use crate::executor::EXECUTOR;
use crate::thread::{Thread, ThreadContext};
use runtime::{Coroutine, CoroutineId, PRIO_NUM};
use spin::Mutex;
use crate::{syscall::*, hart_id, primary_thread};

#[no_mangle]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    // println!("add");
    unsafe { EXECUTOR.as_mut().unwrap() }.add_coroutine(future, prio);
    // println!("add end");
}

#[no_mangle]
pub fn poll_future(a0: usize) {
    let tid = sys_gettid();
    if tid != 0 {
        sleep(50);
    }
    loop {
        match unsafe {EXECUTOR.as_mut().unwrap().fetch()} {
            Some(task) => {
                sleep(10);
                let cid = task.cid;
                match task.execute() {
                    Poll::Pending => {  }
                    Poll::Ready(()) => {
                        unsafe { EXECUTOR.as_mut().unwrap() }.del_coroutine(cid);
                    }
                };
            }
            _ => {
                println!("ex is emtpy");
                break;
            }
        }
    }
    if tid != 0 {
        sys_exit(2);
    }
    sleep(1000);
    // yield_thread(a0);
}

pub fn yield_thread(ctx_addr: usize) {
    unsafe {
        core::arch::asm!(
        r"  .altmacro
            .macro LOAD_SN n
                ld s\n, (\n+2)*8(a0)
            .endm
            
            mv a0, {a0}
            ld ra, 0(a0)
            .set n, 0
            .rept 12
                LOAD_SN %n
                .set n, n + 1
            .endr
            ld sp, 8(a0)
            ret",
            a0  = in(reg) ctx_addr,
            options(noreturn)
        )
    }
}