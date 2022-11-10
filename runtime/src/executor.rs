
use core::task::Waker;

use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::PRIO_NUM;
use crate::{
    coroutine::{Coroutine, CoroutineId},
};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use crate::task_waker::TaskWaker;

pub struct Executor {
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    pub ready_queue: Vec<VecDeque<CoroutineId>>,
    pub waker_cache: BTreeMap<CoroutineId, Arc<Waker>>,
    pub lock: Mutex<usize>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready_queue: Vec::new(),
            waker_cache: BTreeMap::new(),
            lock: Mutex::new(0),
        }
    }
}

impl Executor {
    pub fn add_coroutine(&mut self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
        let cid = CoroutineId::generate();
        let task = Arc::new({
            Coroutine {
                cid,
                future: Mutex::new(future),
                prio,
            }
        });
        let lock = self.lock.lock();
        self.ready_queue[prio].push_back(cid);
        self.tasks.insert(cid, task);
        self.waker_cache.insert(cid, Arc::new(TaskWaker::new(cid, prio)));
        drop(lock);
    }

    pub fn is_empty(&self) -> bool {
        for i in 0..PRIO_NUM {
            if !self.ready_queue[i].is_empty() {
                return false;
            }
        }
        return true;
    }

    pub fn fetch(&mut self) -> (Option<&Arc<Coroutine>>, Option<&Arc<Waker>>) {
        let mut task = None;
        let mut waker = None;
        let lock = self.lock.lock();
        for i in 0..PRIO_NUM {
            if !self.ready_queue[i].is_empty() {
                let cid = self.ready_queue[i].pop_front().unwrap();
                task = self.tasks.get(&cid);
                waker = self.waker_cache.get(&cid);
                break;
            }
        }
        drop(lock);
        (task, waker)
    }

    pub fn del_coroutine(&mut self, cid: CoroutineId) {
        let lock = self.lock.lock();
        self.tasks.remove(&cid);
        self.waker_cache.remove(&cid);
        drop(lock);
    }
}