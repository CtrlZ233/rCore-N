use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::{PRIO_NUM, CoroutineKind};
use crate::{
    coroutine::{Coroutine, CoroutineId},
    BitMap,
};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use crate::config::MAX_THREAD_NUM;

/// 进程 Executor
pub struct Executor {
    /// 当前正在运行的协程 Id
    pub currents: [Option<CoroutineId>; MAX_THREAD_NUM],
    /// 协程 map
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    /// 就绪协程队列
    pub ready_queue: Vec<VecDeque<CoroutineId>>,
    /// 协程优先级位图
    pub bitmap: BitMap,
    /// 进程最高优先级协程代表的优先级，内核可以直接访问物理地址来读取
    pub priority: usize,
    /// 整个 Executor 的读写锁，内核读取 priority 时，可以不获取这个锁，在唤醒协程时，需要获取锁
    pub wr_lock: Mutex<()>,
}

impl Executor {
    /// 
    pub const fn new() -> Self {
        Self {
            currents: [None; MAX_THREAD_NUM],
            tasks: BTreeMap::new(),
            ready_queue: Vec::new(),
            bitmap: BitMap(0),
            priority: PRIO_NUM,
            wr_lock: Mutex::new(()),
        }
    }
}

impl Executor {
    /// 更新协程优先级
    pub fn reprio(&mut self, cid: CoroutineId, prio: usize) {
        let _lock = self.wr_lock.lock();
        let task = self.tasks.get(&cid).unwrap();
        // task.inner.lock().prio = prio;
        let p = task.inner.lock().prio;
        // 先从队列中出来
        if let Ok(idx) = self.ready_queue[p].binary_search(&cid){
            self.ready_queue[p].remove(idx);
            if self.ready_queue[p].is_empty() {
                self.bitmap.update(p, false);
            }
        }
        task.inner.lock().prio = prio;
        self.ready_queue[prio].push_back(cid);
        self.bitmap.update(prio, true);
        self.priority = self.bitmap.get_priority();
    }
    /// 添加协程
    pub fn spawn(&mut self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, kind: CoroutineKind){
        let task = Coroutine::new(future, prio, kind);
        let cid = task.cid;
        let lock = self.wr_lock.lock();
        self.ready_queue[prio].push_back(cid);
        self.tasks.insert(cid, task);
        self.bitmap.update(prio, true);
        if prio < self.priority {
            self.priority = prio;
        }
        drop(lock);
    }
    
    /// 判断是否还有协程
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
    /// 取出优先级最高的协程 id，并且更新位图
    pub fn fetch(&mut self, tid: usize) -> Option<Arc<Coroutine>> {
        assert!(tid < MAX_THREAD_NUM);
        let _lock = self.wr_lock.lock();
        let prio = self.priority;
        if prio == PRIO_NUM {
            self.currents[tid] = None;
            None
        } else {
            let cid = self.ready_queue[prio].pop_front().unwrap();
            let task = (*self.tasks.get(&cid).unwrap()).clone();
            if self.ready_queue[prio].is_empty() {
                self.bitmap.update(prio, false);
                self.priority = self.bitmap.get_priority();
            }
            drop(_lock);
            self.currents[tid] = Some(cid);
            Some(task)
        }
    }
    /// 阻塞协程重新入队
    pub fn re_back(&mut self, cid: CoroutineId) -> usize {
        let lock = self.wr_lock.lock();
        let prio = self.tasks.get(&cid).unwrap().inner.lock().prio;
        self.ready_queue[prio].push_back(cid);
        self.bitmap.update(prio, true);
        if prio < self.priority {
            self.priority = prio;
        }
        drop(lock);
        self.priority
    }
    /// 删除协程，协程已经被执行完了，在 fetch 取出 id 是就已经更新位图了，因此，这时不需要更新位图
    pub fn del_coroutine(&mut self, cid: CoroutineId) {
        let lock = self.wr_lock.lock();
        self.tasks.remove(&cid);
        drop(lock);
    }
}