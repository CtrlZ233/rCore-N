use spin::Mutex;
use super::SimpleMutex;
use alloc::{collections::VecDeque, sync::Arc};

use crate::task::{TaskControlBlock, add_task, TaskContext, current_task, block_current_task, block_current_and_run_next};

pub struct Condvar {
    pub inner: Mutex<CondvarInner>,
}

pub struct CondvarInner {
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Condvar {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                Mutex::new(CondvarInner {
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    pub fn signal(&self) {
        let mut inner = self.inner.lock();
        if let Some(task) = inner.wait_queue.pop_front() {
            add_task(task);
        }
    }

    /*
    pub fn wait(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.wait_queue.push_back(current_task().unwrap());
        drop(inner);
        block_current_and_run_next();
    }
    */

    pub fn wait_no_sched(&self) -> *mut TaskContext {
        let mut inner = self.inner.lock();
        inner.wait_queue.push_back(current_task().unwrap());
        drop(inner);
        block_current_task()
    }

    pub fn wait_with_mutex(&self, mutex: Arc<dyn SimpleMutex>) {
        mutex.unlock();
        let mut inner = self.inner.lock();
        inner.wait_queue.push_back(current_task().unwrap());
        drop(inner);
        block_current_and_run_next();
        mutex.lock();
    }
}