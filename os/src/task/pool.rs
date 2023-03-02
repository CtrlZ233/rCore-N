use alloc::{collections::{BTreeSet, BTreeMap}, sync::Arc};
use lazy_static::*;
use spin::Mutex;

use super::{manager::TaskManager, task::TaskControlBlock, process::ProcessControlBlock};

pub struct TaskPool {
    pub scheduler: TaskManager,
    pub sleeping_tasks: BTreeSet<Arc<TaskControlBlock>>,
}

lazy_static! {
    pub static ref TASK_POOL: Mutex<TaskPool> = Mutex::new(TaskPool::new());
    pub static ref PID2PCB: Mutex<BTreeMap<usize, Arc<ProcessControlBlock>>> = Mutex::new(BTreeMap::new());
}

impl TaskPool {
    pub fn new() -> Self {
        Self {
            scheduler: TaskManager::new(),
            sleeping_tasks: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.add(task);
    }

    pub fn add_user_intr_task(&mut self, pid: usize) {
        self.scheduler.add_user_intr_task(pid);
    }

    pub fn remove_uintr_task(&mut self, pid: usize) {
        self.scheduler.remove_uintr_task(pid);
    }

    #[allow(unused)]
    pub fn remove(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.remove(&task);
    }

    #[allow(unused)]
    pub fn wake(&mut self, task: Arc<TaskControlBlock>) {
        self.sleeping_tasks.remove(&task);
        self.scheduler.add(task);
    }

    #[allow(unused)]
    pub fn sleep(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.remove(&task);
        self.sleeping_tasks.insert(task);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.scheduler.fetch()
    }

    #[allow(unused)]
    pub fn prioritize(&mut self, pid: usize) {
        self.scheduler.prioritize(pid);
    }
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_POOL.lock().add(task);
}

pub fn add_user_intr_task(pid: usize) {
    TASK_POOL.lock().add_user_intr_task(pid);
}

pub fn remove_uintr_task(pid: usize) {
    TASK_POOL.lock().remove_uintr_task(pid);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_POOL.lock().fetch()
}

#[allow(unused)]
pub fn prioritize_task(pid: usize) {
    TASK_POOL.lock().prioritize(pid);
}

pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.lock();
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.lock().insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.lock();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
