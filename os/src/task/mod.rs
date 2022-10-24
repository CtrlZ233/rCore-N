mod context;
mod manager;
mod pid;
mod pool;
mod processor;
mod switch;
mod task;
mod process;

use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;

use spin::Mutex;
use switch::__switch2;

pub use context::TaskContext;
pub use pid::{pid_alloc, KernelStack, PidHandle};
pub use pool::{add_task, fetch_task, prioritize_task};
pub use processor::{
    current_task, current_process, current_trap_cx, current_user_token, hart_id, mmap, munmap, run_tasks, schedule,
    set_current_priority, take_current_task, current_trap_cx_user_va
};
pub use task::{TaskControlBlock, TaskStatus};
use crate::task::pool::remove_from_pid2process;
pub use process::ProcessControlBlock;
use crate::task::pid::TaskUserRes;

lazy_static! {
    pub static ref WAIT_LOCK: Mutex<()> = Mutex::new(());
}

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = current_task().unwrap();
    let mut task_inner = task.acquire_inner_lock();
    task_inner.time_intr_count += 1;
    let task_cx_ptr = task_inner.get_task_cx_ptr();
    drop(task_inner);

    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // ++++++ hold initproc PCB lock here
    // let mut initproc_inner = INITPROC.acquire_inner_lock();

    // take from Processor
    let task = take_current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    // **** hold current PCB lock
    let wl = WAIT_LOCK.lock();
    let mut inner = task.acquire_inner_lock();
    let tid = inner.res.as_ref().unwrap().tid;
    info!(
        "pid: {} exited with code {}, time intr: {}, cycle count: {}",
        task.getpid(), exit_code, inner.time_intr_count, inner.total_cpu_cycle_count
    );
    // if let Some(trap_info) = &inner.user_trap_info {
    //     trap_info.remove_user_ext_int_map();
    //     use riscv::register::sie;
    //     unsafe {
    //         sie::clear_uext();
    //         sie::clear_usoft();
    //         sie::clear_utimer();
    //     }
    // }

    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = Some(exit_code);
    inner.res = None;
    drop(inner);
    // do not move to its parent but under initproc
    if tid == 0 {
        let pid = process.getpid();
        remove_from_pid2process(pid);
        let mut process_inner = process.acquire_inner_lock();
        process_inner.is_zombie = true;
        process_inner.exit_code = exit_code;
        {
            let mut initproc_inner = INITPROC.acquire_inner_lock();
            for child in process_inner.children.iter() {
                child.acquire_inner_lock().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            let mut task_inner = task.acquire_inner_lock();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        recycle_res.clear();
        process_inner.children.clear();
        process_inner.memory_set.recycle_data_pages();
        process_inner.fd_table.clear();
    }

    // **** release current PCB lock
    // drop task manually to maintain rc correctly
    drop(task);
    drop(process);
    drop(wl);
    // we do not have to save task context
    let mut _unused = Default::default();
    schedule(&mut _unused as *mut _);

    // let task = current_task().unwrap();
    // let task_inner = task.acquire_inner_lock();
    // if let Some(trap_info) = &task_inner.user_trap_info {
    //     trap_info.enable_user_ext_int();
    // }
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> =
        ProcessControlBlock::new(get_app_data_by_name("initproc").unwrap());
}

pub fn add_initproc() {
    debug!("add_initproc");
    let _initproc = INITPROC.clone();
}
