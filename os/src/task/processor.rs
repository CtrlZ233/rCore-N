use super::TaskContext;
use super::TaskControlBlock;
use super::__switch2;
use super::add_task;
use super::{fetch_task, TaskStatus};
use crate::config::CPU_NUM;
use crate::trace::SCHEDULE;
use crate::trace::{push_trace, RUN_NEXT, SUSPEND_CURRENT};
use crate::trap::TrapContext;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use core::cell::RefCell;
use riscv::register::cycle;

use lazy_static::*;
use crate::task::process::ProcessControlBlock;
use crate::task::suspend_current_and_run_next;
lazy_static! {
    pub static ref PROCESSORS: [Processor; CPU_NUM] = Default::default();
}

pub struct Processor {
    inner: RefCell<ProcessorInner>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx: Default::default(),
                idle_task_cx_ptr: 0,
            }),
        }
    }
}

unsafe impl Sync for Processor {}

struct ProcessorInner {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
    idle_task_cx_ptr: usize,
}

impl Processor {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx: Default::default(),
                idle_task_cx_ptr: 0,
            }),
        }
    }
    fn get_idle_task_cx_ptr(&self) -> *mut TaskContext {
        let mut inner = self.inner.borrow_mut();
        &mut inner.idle_task_cx as *mut TaskContext
    }
    #[allow(unused)]
    #[deprecated]
    fn get_idle_task_cx_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_task_cx_ptr as *const usize
    }

    fn run_next(&self, task: Arc<TaskControlBlock>) {
        push_trace(RUN_NEXT + task.getpid());
        let idle_task_cx_ptr = self.get_idle_task_cx_ptr();
        trace!(
            "[run next] idle task cx ptr: {:x?}, task cx: {:#x?}",
            idle_task_cx_ptr,
            unsafe { &*idle_task_cx_ptr }
        );
        // acquire
        let process = task.process.upgrade().unwrap();
        let process_inner = process.acquire_inner_lock();
        if process_inner.is_user_trap_enabled() {
            process_inner.user_trap_info.as_ref().unwrap().enable_user_ext_int();
        }

        drop(process_inner);
        drop(process);

        let mut task_inner = task.acquire_inner_lock();
        let next_task_cx_ptr = task_inner.get_task_cx_ptr();
        task_inner.task_status = TaskStatus::Running(hart_id());

        let task_cx = unsafe { &*next_task_cx_ptr };
        trace!(
            "next task cx ptr: {:#x?}, task cx: {:#x?}",
            next_task_cx_ptr,
            task_cx
        );
        task_inner.last_cpu_cycle = cycle::read();
        // release
        drop(task_inner);
        self.inner.borrow_mut().current = Some(task);
        unsafe {
            __switch2(idle_task_cx_ptr, next_task_cx_ptr);
        }
    }

    fn suspend_current(&self) {
        trace!("[suspend current]");
        if let Some(task) = take_current_task() {
            // ---- hold current PCB lock
            push_trace(SUSPEND_CURRENT + task.getpid());
            let process = task.process.upgrade().unwrap();
            let process_inner = process.acquire_inner_lock();
            if process_inner.is_user_trap_enabled() {
                process_inner.user_trap_info.as_ref().unwrap().disable_user_ext_int();
            }
            drop(process_inner);
            drop(process);
            // Change status to Ready
            let mut task_inner = task.acquire_inner_lock();
            task_inner.task_status = TaskStatus::Ready;
            task_inner.total_cpu_cycle_count += cycle::read() - task_inner.last_cpu_cycle;
            drop(task_inner);
            // ---- release current PCB lock
            // push back to ready queue.
            add_task(task);
        }
    }

    // pub fn run(&self) {
    //     loop {
    //         if hart_id() == 0 {
    //             unifi_exposure::poll_kernel_future();
    //         }
    //         if let Some(task) = fetch_task() {
    //             self.run_next(task);
    //             self.suspend_current();
    //         }
    //     }
    // }
    pub fn take_current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow_mut().current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner
            .borrow()
            .current
            .as_ref()
            .map(|task| Arc::clone(task))
    }
}

// lazy_static! {
//     pub static ref PROCESSOR: Processor = Processor::new();
// }

#[inline]
pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

// pub fn run_tasks() {
//     debug!("run_tasks");
//     PROCESSORS[hart_id()].run();
// }
pub async fn run_tasks() {
    let mut helper = Box::new(ReadHelper::new());
    loop {
        if let Some(task) = fetch_task() {
            PROCESSORS[hart_id()].run_next(task);
            PROCESSORS[hart_id()].suspend_current();
        }
        helper.as_mut().await;
    }

}
use core::{task::{Context, Poll}, future::Future, pin::Pin};
use alloc::boxed::Box;
pub struct ReadHelper(usize);

impl ReadHelper {
    pub fn new() -> Self {
        Self(0)
    }
}
impl Future for ReadHelper {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0 += 1;
        if (self.0 & 1) == 1 {
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSORS[hart_id()].take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSORS[hart_id()].current()
}

pub fn current_process() -> Option<Arc<ProcessControlBlock>> {
    current_task().unwrap().process.upgrade()
}

#[allow(unused)]
pub fn current_tasks() -> Vec<Option<Arc<TaskControlBlock>>> {
    PROCESSORS
        .iter()
        .map(|processor| processor.current())
        .collect()
}

pub fn current_user_token() -> usize {
    let process = current_process().unwrap();
    let token = process.acquire_inner_lock().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().acquire_inner_lock().get_trap_cx()
}

pub fn current_trap_cx_user_va() -> usize {
    current_task()
        .unwrap()
        .acquire_inner_lock()
        .res
        .as_ref()
        .unwrap()
        .trap_cx_user_va()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    push_trace(SCHEDULE);
    let idle_task_cx_ptr = PROCESSORS[hart_id()].get_idle_task_cx_ptr();
    trace!(
        "[schedule] switched task cx ptr: {:x?}, task cx: {:x?}",
        switched_task_cx_ptr,
        unsafe { &*switched_task_cx_ptr }
    );
    trace!(
        "[schedule] idle task cx ptr: {:x?}, task cx: {:x?}",
        idle_task_cx_ptr,
        unsafe { &*idle_task_cx_ptr }
    );
    unsafe {
        __switch2(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

pub fn set_current_priority(priority: isize) -> Result<isize, isize> {
    if let Some(current) = current_task() {
        let mut current = current.acquire_inner_lock();
        current.set_priority(priority)
    } else {
        Err(-1)
    }
}

pub fn mmap(start: usize, len: usize, port: usize) -> Result<isize, isize> {
    if let Some(current) = current_process() {
        let mut current = current.acquire_inner_lock();
        current.mmap(start, len, port)
    } else {
        Err(-1)
    }
}

pub fn munmap(start: usize, len: usize) -> Result<isize, isize> {
    if let Some(current) = current_process() {
        let mut current = current.acquire_inner_lock();
        current.munmap(start, len)
    } else {
        Err(-1)
    }
}
