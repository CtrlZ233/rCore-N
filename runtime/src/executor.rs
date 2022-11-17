

use alloc::collections::{BTreeMap, VecDeque, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::PRIO_NUM;
use crate::{
    coroutine::{Coroutine, CoroutineId},
    BitMap,
};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;

pub struct Executor {
    pub current: Option<CoroutineId>,
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    pub ready_queue: Vec<VecDeque<CoroutineId>>,
    // 协程优先级位图
    pub bitmap: BitMap,
    // 进程最高优先级协程代表的优先级，内核可以直接访问物理地址来读取
    pub priority: usize,
    // callback_queue 没有必要单独加锁，整个 Executor 粒度的锁足够了
    pub callback_queue: Vec<CoroutineId>,
    // 被唤醒过的协程的 id 保存在这里
    pub callback_cid: BTreeSet<CoroutineId>,
    // 整个 Executor 的读写锁，内核读取 priority 时，可以不获取这个锁，在唤醒协程时，需要获取锁
    pub wr_lock: Mutex<()>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            current: None,
            tasks: BTreeMap::new(),
            ready_queue: Vec::new(),
            bitmap: BitMap(0),
            priority: PRIO_NUM,
            callback_queue: Vec::new(),
            callback_cid: BTreeSet::new(),
            wr_lock: Mutex::new(()),
        }
    }
}

impl Executor {
    pub fn add_coroutine(&mut self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
        let lock = self.wr_lock.lock();
        let task = Coroutine::new(Mutex::new(future), prio);
        let cid = task.cid;
        self.ready_queue[prio].push_back(cid);
        self.tasks.insert(cid, task);
        self.bitmap.update(prio, true);
        if prio < self.priority {
            self.priority = prio;
        }
        drop(lock);
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    // 取出优先级最高的协程 id，并且更新位图
    pub fn fetch(&mut self) -> Option<Arc<Coroutine>> {
        let _lock = self.wr_lock.lock();
        // 唤醒回调队列中的任务
        while self.callback_queue.len() != 0 {
            if let Some(id) = self.callback_queue.pop() {
                self.callback_cid.insert(id);
                let prio = self.tasks.get(&id).unwrap().prio;
                self.ready_queue[prio].push_back(id);
                self.bitmap.update(prio, true);
                if prio < self.priority {
                    self.priority = prio;
                }
            }
        }
        let prio = self.priority;
        if prio == PRIO_NUM {
            self.current = None;
            None
        } else {
            let cid = self.ready_queue[prio].pop_front().unwrap();
            let task = (*self.tasks.get(&cid).unwrap()).clone();
            if self.ready_queue[prio].is_empty() {
                self.bitmap.update(prio, false);
                self.priority = self.bitmap.get_priority();
            }
            self.current = Some(cid);
            Some(task)
        }
    }

    // 删除协程，协程已经被执行完了，在 fetch 取出 id 是就已经更新位图了，因此，这时不需要更新位图
    pub fn del_coroutine(&mut self, cid: CoroutineId) {
        let lock = self.wr_lock.lock();
        self.tasks.remove(&cid);
        drop(lock);
    }

    // 判断协程是否被唤醒过
    pub fn is_waked(&mut self, cid: CoroutineId) -> bool {
        self.callback_cid.contains(&cid)
    }
}