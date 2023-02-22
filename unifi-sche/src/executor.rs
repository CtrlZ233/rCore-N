
use unifi_exposure::{Executor, CoroutineId};
use crate::{config::UNFI_SCHE_BUFFER, prio_array::max_prio_pid};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::sync::atomic::Ordering;
use crate::prio_array::{update_prio, PRIO_ARRAY};
use syscall::*;
use core::task::Poll;
use crate::MAX_THREAD_NUM;
use buddy_system_allocator::LockedHeap;



/// 添加协程，内核和用户态都可以调用
pub fn spawn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) {
    unsafe {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        (*exe).spawn(future, prio);
        // 更新优先级标记
        let prio = (*exe).priority;
        update_prio(pid, prio);
    }
}
/// 用户程序执行协程
pub fn poll_user_future() {
    unsafe {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let pid = getpid() as usize;
        let tid = gettid();
        loop {
            if (*exe).is_empty() {
                println!("ex is empty");
                break;
            }
            let task = (*exe).fetch(tid as usize);
            // 每次取出协程之后，需要更新优先级标记
            let prio = (*exe).priority;
            update_prio(pid + 1, prio);
            match task {
                Some(task) => {
                    let cid = task.cid;
                    match task.execute() {
                        Poll::Pending => { }
                        Poll::Ready(()) => {
                            (*exe).del_coroutine(cid);
                        }
                    };
                }
                _ => {
                    // 任务队列不为空，但就绪队列为空，等待任务唤醒
                    yield_();
                }
            }
            // 执行完优先级最高的协程，检查优先级，判断是否让权
            let max_prio_pid = max_prio_pid();
            if pid + 1 != max_prio_pid {
                yield_();
            }
        }
        if tid != 0 {
            exit(2);
        }
    }
}
/// 内核执行协程
pub fn poll_kernel_future() {
    unsafe {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        loop {
            let task = (*exe).fetch(0);
            // 更新优先级标记
            let prio = (*exe).priority;
            update_prio(0, prio);
            match task {
                Some(task) => {
                    let cid = task.cid;
                    match task.execute() {
                        Poll::Pending => {
                        }
                        Poll::Ready(()) => {
                            (*exe).del_coroutine(cid);
                        }
                    };
                }
                _ => {
                    break;
                }
            }
        }
    }
}
/// 获取当前正在执行的协程 id
pub fn current_cid(is_kernel: bool) -> usize {
    let tid = if is_kernel { 0 } else {
        gettid() as usize
    };
    assert!(tid < MAX_THREAD_NUM);
    unsafe {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        (*exe).currents[tid].as_mut().unwrap().get_val()
    }
}

/// 协程重新入队，手动执行唤醒的过程，内核和用户都会调用这个函数
pub fn re_back(cid: usize, pid: usize) {
    // println!("[Exec]re back func enter");
    unsafe {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let prio = (*exe).re_back(CoroutineId(cid));
        // 重新入队之后，需要检查优先级
        let process_prio = PRIO_ARRAY[pid].load(Ordering::Relaxed);
        if prio < process_prio {
            PRIO_ARRAY[pid].store(prio, Ordering::Relaxed);
        }
    }
}

/// 更新协程优先级
pub fn reprio(cid: usize, prio: usize) {
    unsafe {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        (*exe).reprio(CoroutineId(cid), prio);
    }
}
