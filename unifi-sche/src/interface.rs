
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::sync::atomic::Ordering;
use crate::executor::Exe;
use unifi_exposure::{MAX_PROC_NUM, CoroutineId};
use core::sync::atomic::AtomicUsize;

#[no_mangle]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize){
    Exe::add_coroutine(future, prio, pid);    
}

#[no_mangle]
pub fn current_cid() -> usize {
    Exe::current_cid()    
}

#[no_mangle]
pub fn poll_future() {
    Exe::poll_user_future();
}

#[no_mangle]
pub fn re_back(cid: usize, pid: usize) {
    println!("re back func enter");
    Exe::re_back(cid, pid);
}

#[no_mangle]
pub fn poll_kernel_future() {
    Exe::poll_kernel_future();
}


// 各个进程的最高优先级协程，通过共享内存的形式进行通信
pub static mut PRIO_ARRAY: [AtomicUsize; MAX_PROC_NUM + 1] = [const { AtomicUsize::new(usize::MAX) }; MAX_PROC_NUM + 1];

// 进程内的线程通过原子操作互斥更新
pub fn update_prio(idx: usize, prio: usize) {
    unsafe {
        PRIO_ARRAY[idx].store(prio, Ordering::Relaxed);
    }
}

// 内核在发生时中中断，重新调度进程时，调用这个函数，选出进程，再选出对应的线程
pub fn max_prio_pid() -> usize {
    let mut ret;
    let mut pid = 0;
    unsafe {
        ret = PRIO_ARRAY[0].load(Ordering::Relaxed);
    }

    for i in 1..MAX_PROC_NUM {
        unsafe {
            let prio = PRIO_ARRAY[i].load(Ordering::Relaxed);
            if prio < ret {
                ret = prio;
                pid = i;
            }
        }
    }
    pid
}

