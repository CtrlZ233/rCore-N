//! 共享调度器模块

#![no_std]
#![no_main]
#![feature(inline_const)]

#[macro_use]
extern crate lib_so;
extern crate alloc;

use lib_so::config::{ENTRY, MAX_THREAD_NUM, MAX_PROC_NUM, HEAP_BUFFER};
use spin::Mutex;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicUsize;
use lib_so::{Executor, CoroutineId, CoroutineKind};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use syscall::*;
use core::task::Poll;
use buddy_system_allocator::Heap;
type LockedHeap = Mutex<Heap>;


// 自定义的模块接口，模块添加进地址空间之后，需要执行 _start() 函数填充这个接口表
static mut INTERFACE: [usize; 10] = [0; 10];

#[no_mangle]
fn main() -> usize{
    unsafe {
        INTERFACE[0] = user_entry as usize;
        INTERFACE[1] = max_prio_pid as usize;
        INTERFACE[2] = spawn as usize;
        INTERFACE[3] = poll_kernel_future as usize;
        INTERFACE[4] = re_back as usize;
        INTERFACE[5] = current_cid as usize;
        INTERFACE[6] = reprio as usize;
        INTERFACE[7] = add_virtual_core as usize;
        INTERFACE[8] = update_prio as usize;
        INTERFACE[9] = get_pending_status as usize;
        &INTERFACE as *const [usize; 10] as usize
    }
}

/// sret 进入用户态的入口，在这个函数再执行 main 函数
#[no_mangle]
#[inline(never)]
fn user_entry() {
    unsafe {
        let secondary_init: fn(usize) = core::mem::transmute(ENTRY);
        // main_addr 表示用户进程 main 函数的地址
        secondary_init(&INTERFACE as *const [usize; 10] as usize);
    }
    let start = get_time();

    poll_user_future();
    wait_other_cores();

    let end = get_time();
    println!("total time: {} ms", end - start);
    
    exit(0);
}


/// 各个进程的最高优先级协程，通过共享内存的形式进行通信
pub static mut PRIO_ARRAY: [AtomicUsize; MAX_PROC_NUM + 1] = [const { AtomicUsize::new(usize::MAX) }; MAX_PROC_NUM + 1];

/// 进程的 Executor 调用这个函数，通过原子操作更新自己的最高优先级
#[no_mangle]
#[inline(never)]
pub fn update_prio(idx: usize, prio: usize) {
    unsafe {
        PRIO_ARRAY[idx].store(prio, Ordering::Relaxed);
    }
}

/// 内核重新调度进程时，调用这个函数，选出优先级最高的进程，再选出对应的线程
/// 所有进程的优先级相同时，则内核会优先执行协程，这里用 0 来表示内核的优先级
#[no_mangle]
#[inline(never)]
pub fn max_prio_pid() -> usize {
    let mut ret;
    let mut pid = 1;
    unsafe {
        ret = PRIO_ARRAY[1].load(Ordering::Relaxed);
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


/// 添加协程，内核和用户态都可以调用
#[no_mangle]
#[inline(never)]
pub fn spawn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize, kind: CoroutineKind) -> usize {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let cid = (*exe).spawn(future, prio, kind);
        // 更新优先级标记
        let prio = (*exe).priority;
        update_prio(pid, prio);
        // if pid == 0 {
        //     println_hart!("executor prio {}", hart_id(), prio);
        // } else {
        //     println!("executor prio {}", prio);
        // }
        return cid;
    }
}


/// 用户程序执行协程
#[no_mangle]
#[inline(never)]
pub fn poll_user_future() {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let pid = getpid() as usize;
        let tid = gettid();
        loop {
            if (*exe).is_empty() {
                // println!("ex is empty");
                break;
            }
            let task = (*exe).fetch(tid as usize);
            match task {
                Some(task) => {
                    let cid = task.cid;
                    // println!("user task kind {:?}", task.kind);
                    match task.execute() {
                        Poll::Pending => {
                            (*exe).pending(cid.0);
                        }
                        Poll::Ready(()) => {
                            (*exe).del_coroutine(cid);
                        }
                    };
                    {
                        let _lock = (*exe).wr_lock.lock();
                        let prio: usize = (*exe).priority;
                        update_prio(getpid() as usize + 1, prio);
                    }
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
/// hart_id
#[allow(unused)]
pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}
/// 内核执行协程
#[no_mangle]
#[inline(never)]
pub fn poll_kernel_future() {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        loop {
            let task = (*exe).fetch(hart_id());
            // 更新优先级标记
            let prio = (*exe).priority;
            update_prio(0, prio);
            // println_hart!("executor prio {}", hart_id(), prio);
            match task {
                Some(task) => {
                    let cid = task.cid;
                    let kind = task.kind;
                    let _prio = task.inner.lock().prio;
                    match task.execute() {
                        Poll::Pending => {
                            (*exe).pending(cid.0);
                            if kind == CoroutineKind::KernSche {
                                // println_hart!("pending reback sche task{:?} kind {:?}", hart_id(), cid, kind);
                                re_back(cid.0, 0);
                            }
                        }
                        Poll::Ready(()) => {
                            (*exe).del_coroutine(cid);
                        }
                    };
                }
                _ => {
                }
            }
        }
    }
}
/// 获取当前正在执行的协程 id
#[no_mangle]
#[inline(never)]
pub fn current_cid(is_kernel: bool) -> usize {
    let tid = if is_kernel { hart_id() } else {
        gettid() as usize
    };
    assert!(tid < MAX_THREAD_NUM);
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        (*exe).currents[tid].as_mut().unwrap().get_val()
    }
}

/// 协程重新入队，手动执行唤醒的过程，内核和用户都会调用这个函数
#[no_mangle]
#[inline(never)]
pub fn re_back(cid: usize, pid: usize) {
    // println!("[Exec]re back func enter");
    let mut start = 0;
    
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let prio = (*exe).re_back(CoroutineId(cid));
        // 重新入队之后，需要检查优先级
        let process_prio = PRIO_ARRAY[pid].load(Ordering::Relaxed);
        if prio < process_prio {
            PRIO_ARRAY[pid].store(prio, Ordering::Relaxed);
        }
    }
}

#[no_mangle]
#[inline(never)]
pub fn get_pending_status(cid: usize) -> bool {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        return (*exe).is_pending(cid)
    }
}

/// 更新协程优先级
#[no_mangle]
#[inline(never)]
pub fn reprio(cid: usize, prio: usize) {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        (*exe).reprio(CoroutineId(cid), prio);
    }
}

/// 申请虚拟CPU
#[no_mangle]
#[inline(never)]
pub fn add_virtual_core() {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let tid = thread_create(poll_user_future as usize, 0) as usize;
        (*exe).add_wait_tid(tid);
    }
}


pub fn wait_other_cores() {
    unsafe {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        for tid in (*exe).waits.iter() {
            waittid(*tid);
        }
    }
}
