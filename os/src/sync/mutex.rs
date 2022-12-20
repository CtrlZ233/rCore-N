use spin::Mutex;
use crate::task::{suspend_current_and_run_next, TaskControlBlock, current_task,
    block_current_and_run_next, add_task};
use alloc::{collections::VecDeque,sync::Arc};
pub trait SimpleMutex: Sync + Send {
    fn lock(&self);
    fn unlock(&self);
}

pub struct MutexSpin {
    locked: spin::Mutex<bool>,
}

impl MutexSpin {
    pub fn new() -> Self {
        Self {
            locked: unsafe { Mutex::new(false) },
        }
    }
}

impl SimpleMutex for MutexSpin {
    fn lock(&self) {
        loop {
            let mut locked = self.locked.lock();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        let mut locked = self.locked.lock();
        *locked = false;
    }
}

pub struct MutexBlocking {
    inner: Mutex<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}


impl MutexBlocking {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                Mutex::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl SimpleMutex for MutexBlocking {
    fn lock(&self) {
        let mut mutex_inner = self.inner.lock();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner.lock();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
