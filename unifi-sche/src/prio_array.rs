use core::sync::atomic::Ordering;
use unifi_exposure::MAX_PROC_NUM;
use core::sync::atomic::AtomicUsize;

/// 各个进程的最高优先级协程，通过共享内存的形式进行通信
pub static mut PRIO_ARRAY: [AtomicUsize; MAX_PROC_NUM + 1] = [const { AtomicUsize::new(usize::MAX) }; MAX_PROC_NUM + 1];

/// 进程的 Executor 调用这个函数，通过原子操作更新自己的最高优先级
pub fn update_prio(idx: usize, prio: usize) {
    unsafe {
        PRIO_ARRAY[idx].store(prio, Ordering::Relaxed);
    }
}

/// 内核重新调度进程时，调用这个函数，选出优先级最高的进程，再选出对应的线程
/// 所有进程的优先级相同时，则内核会优先执行协程，这里用 0 来表示内核的优先级
pub fn max_prio_pid() -> usize {
    let mut ret;
    let mut pid = 1;
    unsafe {
        ret = PRIO_ARRAY[1].load(Ordering::Relaxed);
    }
    for i in 2..MAX_PROC_NUM {
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

