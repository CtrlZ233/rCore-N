
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::sync::atomic::Ordering;
use core::task::Poll;
use crate::executor::EXECUTOR;
use crate::syscall::*;
use crate::config::MAX_PROC_NUM;
use core::sync::atomic::AtomicUsize;

#[no_mangle]
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    // println!("add");
    unsafe { 
        EXECUTOR.as_mut().unwrap().add_coroutine(future, prio);

        let prio = EXECUTOR.as_mut().unwrap().priority;
        let pid = sys_getpid() as usize;
        update_prio(pid, prio);
    }
    
    // println!("add end");
}

#[no_mangle]
pub fn poll_future(a0: usize) {
    let tid = sys_gettid();
    if tid != 0 {
        sleep(50);
    }
    loop {
        let task = unsafe { EXECUTOR.as_mut().unwrap().fetch() };
        let prio = unsafe { EXECUTOR.as_mut().unwrap().priority };
        let pid = sys_getpid() as usize;
        update_prio(pid, prio);
        match task {
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

// 各个进程的最高优先级协程，通过共享内存的形式进行通信
pub static mut PRIO_ARRAY: [AtomicUsize; MAX_PROC_NUM] = [const { AtomicUsize::new(usize::MAX) }; MAX_PROC_NUM];

// 进程内的线程通过原子操作互斥更新
pub fn update_prio(idx: usize, prio: usize) {
    unsafe {
        PRIO_ARRAY[idx].store(prio, Ordering::Relaxed);
    }
}

// 内核在发生时中中断，重新调度进程时，调用这个函数，选出进程，再选出对应的线程
pub fn max_prio_pid() -> usize {
    let mut ret = usize::MAX;
    let mut pid = 0;
    for i in 0..MAX_PROC_NUM {
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

