
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::task::{Poll, Context};
use alloc::sync::Arc;
use crate::executor::EXECUTOR;
use crate::thread::{Thread, ThreadContext};
use runtime::{Coroutine, CoroutineId, PRIO_NUM, TaskWaker};
use spin::Mutex;
use crate::{syscall::*, hart_id, primary_thread};

#[no_mangle]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    unsafe {
        let ex = EXECUTOR[hart_id()].as_mut().unwrap();
        ex.add_coroutine(future, prio);
    }
}

#[no_mangle]
pub fn poll_future(a0: usize) {
    loop {
        let ex = unsafe { EXECUTOR[hart_id()].as_mut().unwrap() };
        if ex.is_empty() {
            println!("ex is empty");
            break;
        }
        let (task, waker) = ex.fetch();
        let cid = task.unwrap().cid;
        let mut context = Context::from_waker(&*waker.unwrap());
        let mut can_delete = false;
        match task.unwrap().future.lock().as_mut().poll(&mut context) {
            Poll::Pending => {  }
            Poll::Ready(()) => {
                can_delete = true;
            }
        };
        if can_delete {
            ex.del_coroutine(cid);
        }
    }
    yield_thread(a0);
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