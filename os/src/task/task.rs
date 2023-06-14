use super::TaskContext;
use super::{pid_alloc, KernelStack, PidHandle};
use crate::fs::{File, MailBox, Serial, Socket, Stdin, Stdout};
use crate::mm::{translate_writable_va, MemorySet, PhysAddr, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::task::pid::{kstack_alloc, RecycleAllocator, TaskUserRes};
use crate::trap::{trap_handler, TrapContext, UserTrapInfo, UserTrapQueue};
use crate::{
    config::{PAGE_SIZE, TRAP_CONTEXT, USER_TRAP_BUFFER},
    loader::get_app_data_by_name,
    mm::translated_str,
};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use spin::{Mutex, MutexGuard};
use crate::task::process::ProcessControlBlock;


pub struct TaskControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: Mutex<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub priority: isize,
    pub exit_code: Option<i32>,
    pub mail_box: Arc<MailBox>,
    pub time_intr_count: usize,
    pub total_cpu_cycle_count: usize,
    pub last_cpu_cycle: usize,
    pub interrupt_time: usize,
    pub user_time_us: usize,
    pub last_user_time_us: usize
}

impl TaskControlBlockInner {
    pub fn get_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.task_cx as *mut TaskContext
    }
    #[deprecated]
    #[allow(unused)]
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }

    pub fn set_priority(&mut self, priority: isize) -> Result<isize, isize> {
        if priority < 2 {
            return Err(-1);
        }
        self.priority = priority;
        Ok(priority)
    }

    pub fn is_mailbox_full(&self) -> bool {
        self.mail_box.is_full()
    }

    pub fn is_mailbox_empty(&self) -> bool {
        self.mail_box.is_empty()
    }
}

impl TaskControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }

    pub fn try_acquire_inner_lock(&self) -> Option<MutexGuard<TaskControlBlockInner>> {
        self.inner.try_lock()
    }
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.acquire_inner_lock();
        inner.memory_set.token()
    }

    pub fn getpid(&self) -> usize {
        self.process.upgrade().unwrap().getpid()
    }

    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let tid = res.tid;
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: Mutex::new(
                TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top, tid),
                    task_cx_ptr: 0,
                    task_status: TaskStatus::Ready,
                    priority: 0,
                    exit_code: None,
                    mail_box: Arc::new(MailBox::new()),
                    time_intr_count: 0,
                    total_cpu_cycle_count: 0,
                    last_cpu_cycle: 0,
                    interrupt_time: 0,
                    user_time_us: 0,
                    last_user_time_us: 0,
                }
            )
        }

    }

    #[allow(unused)]
    pub fn create_socket(&self) -> Arc<Socket> {
        self.inner.lock().mail_box.create_socket()
    }
}

impl PartialEq for TaskControlBlock {
    fn eq(&self, other: &Self) -> bool {
        self.getpid() == other.getpid()
    }
}

impl Eq for TaskControlBlock {}

impl PartialOrd for TaskControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskControlBlock {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.getpid().cmp(&other.getpid())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running(usize),
    Zombie,
    Blocking,
}
