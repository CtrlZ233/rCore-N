
use unifi_exposure::{Executor, CoroutineId};
use crate::heap::MutAllocator;
use spin::Mutex;
use crate::config::UNFI_SCHE_BUFFER;
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use core::sync::atomic::Ordering;
use crate::interface::{update_prio, PRIO_ARRAY};
use syscall::*;
use core::task::Poll;

pub struct Exe;

impl Exe {
    pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) {
        unsafe {
            let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
            let exe = (heapptr + core::mem::size_of::<Mutex<MutAllocator<32>>>()) as *mut usize as *mut Executor;
            (*exe).add_coroutine(future, prio);
            let prio = (*exe).priority;
            update_prio(pid, prio);
        }
    }

    pub fn poll_user_future() {
        unsafe {
            let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
            let exe = (heapptr + core::mem::size_of::<Mutex<MutAllocator<32>>>()) as *mut usize as *mut Executor;
            let tid = gettid();
            if tid != 0 {
                sleep(50);
            }
            loop {
                if (*exe).is_empty() {
                    println!("ex is empty");
                    break;
                }
                let task = (*exe).fetch();
                let prio = (*exe).priority;
                let pid = getpid() as usize;
                update_prio(pid + 1, prio);
                match task {
                    Some(task) => {
                        sleep(10);
                        let cid = task.cid;
                        match task.execute() {
                            Poll::Pending => {  }
                            Poll::Ready(()) => {
                                (*exe).del_coroutine(cid);
                            }
                        };
                    }
                    _ => { }
                }
            }
            if tid != 0 {
                exit(2);
            }
            sleep(1000);
        }
    }

    pub fn poll_kernel_future() {
        unsafe {
            let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
            let exe = (heapptr + core::mem::size_of::<Mutex<MutAllocator<32>>>()) as *mut usize as *mut Executor;
            loop {
                let task = (*exe).fetch();
                // let prio = (*exe).priority;
                // update_prio(0, prio);
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

    pub fn current_cid() -> usize {
        unsafe {
            let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
            let exe = (heapptr + core::mem::size_of::<Mutex<MutAllocator<32>>>()) as *mut usize as *mut Executor;
            (*exe).current.as_mut().unwrap().get_val()
        }
    }


    pub fn re_back(cid: usize, pid: usize) {
        println!("[Exec]re back func enter");
        unsafe {
            let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
            let exe = (heapptr + core::mem::size_of::<Mutex<MutAllocator<32>>>()) as *mut usize as *mut Executor;

            let prio = (*exe).re_back(CoroutineId(cid));
            let process_prio = PRIO_ARRAY[pid].load(Ordering::Relaxed);
            if prio < process_prio {
                PRIO_ARRAY[pid].store(prio, Ordering::Relaxed);
            }
        }
    }
}