
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
    let cid = CoroutineId::generate();
    let task = Arc::new({
        Coroutine {
            cid,
            future: Mutex::new(future),
            prio,
        }
    });
    unsafe {
        let ex = EXECUTOR[hart_id()].as_mut().unwrap();
        let lock = ex.lock.lock();
        ex.ready_queue[prio].push_back(cid);
        ex.tasks.insert(cid, task);
        ex.waker_cache.insert(cid, Arc::new(TaskWaker::new(cid, prio)));
        drop(lock);
    }
}

#[no_mangle]
pub fn run(){
    // let ex = unsafe { EXECUTOR.as_mut().unwrap() };
    // if ex.is_empty() { 
    //     println!("ex is empty");
    //     sys_exit(0); 
    // }
    // let mut thread = Thread::new();
    // thread.execute();
}

#[no_mangle]
pub fn poll_future(a0: usize) {
    loop {
        let ex = unsafe { EXECUTOR[hart_id()].as_mut().unwrap() };
        let lock = ex.lock.lock();
        let mut flag = false;
        let mut cid = CoroutineId(usize::MAX);
        let mut task = None;
        {
            for i in 0..PRIO_NUM {
                if !ex.ready_queue[i].is_empty(){
                    cid = ex.ready_queue[i].pop_front().unwrap();
                    task = ex.tasks.get(&cid);
                    flag = true;
                    break;
                }
            }
        }
        if !flag {
            println!("ex is empty");
            break;
        }
        let waker = ex.waker_cache.get(&cid).unwrap();
        let mut context = Context::from_waker(&*waker);
        let mut can_delete = false;
        drop(lock);
        match task.unwrap().future.lock().as_mut().poll(&mut context) {
            Poll::Pending => {  }
            Poll::Ready(()) => {
                // remove task
                can_delete = true;
            }
        };
        let lock = ex.lock.lock();
        if can_delete {
            ex.tasks.remove(&cid);
            ex.waker_cache.remove(&cid);
        }
        drop(lock);
    }
    yield_thread(a0);
}

pub fn yield_thread(ctx_addr: usize) {
    let ctx = ctx_addr as *const ThreadContext;
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