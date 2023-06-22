use alloc::collections::{BTreeMap, VecDeque, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use syscall::yield_;
use super::{
    coroutine::{Coroutine, CoroutineId, CoroutineKind},
    BitMap,
};
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use crate::config::{MAX_THREAD_NUM, PRIO_NUM};

pub struct ExMutex {
    mutex: Mutex<()>,
    busy_wait: bool,
}

impl ExMutex {
    pub const fn new(busy_wait: bool) -> Self {
        ExMutex { mutex: Mutex::new(()), busy_wait, }
    }

    pub fn lock(&mut self) -> spin::MutexGuard<'_, ()> {
        if self.busy_wait {
            self.mutex.lock()
        } else {
            loop {
                let mut op_lock = self.mutex.try_lock();
                if op_lock.is_some() {
                    return op_lock.unwrap();
                }
                yield_();
            }
            
        }
    }
}

/// 进程 Executor
pub struct Executor {
    /// 当前正在运行的协程 Id
    pub currents: [Option<CoroutineId>; MAX_THREAD_NUM],
    /// 协程 map
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    /// 就绪协程队列
    pub ready_queue: Vec<VecDeque<CoroutineId>>,
    /// 阻塞协程集合
    pub pending_set: BTreeSet<usize>,
    /// 协程优先级位图
    pub bitmap: BitMap,
    /// 进程最高优先级协程代表的优先级，内核可以直接访问物理地址来读取
    pub priority: usize,
    /// 整个 Executor 的读写锁，内核读取 priority 时，可以不获取这个锁，在唤醒协程时，需要获取锁
    pub wr_lock: ExMutex,
    /// 执行器线程id
    pub waits: Vec<usize>,
}

impl Executor {
    /// 
    pub const fn new(busy_wait: bool) -> Self {
        Self {
            currents: [None; MAX_THREAD_NUM],
            tasks: BTreeMap::new(),
            ready_queue: Vec::new(),
            pending_set: BTreeSet::new(),
            bitmap: BitMap(0),
            priority: PRIO_NUM,
            wr_lock: ExMutex::new(busy_wait),
            waits: Vec::new(),
        }
    }
}

impl Executor {
    /// 更新协程优先级
    pub fn reprio(&mut self, cid: CoroutineId, prio: usize) {
        let _lock: spin::MutexGuard<'_, ()> = self.wr_lock.lock();
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
    pub fn spawn(&mut self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, kind: CoroutineKind) -> usize {
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
        return cid.0;
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

    // /// 取出优先级最高的协程 id，并且更新位图
    // pub fn fetch_user(&mut self, tid: usize) -> Option<Arc<Coroutine>> {
    //     assert!(tid < MAX_THREAD_NUM);
    //     let mut op_lock;
    //     let mut _lock;
    //     while true {
    //         op_lock = self.wr_lock.try_lock();
    //         if op_lock.is_some() {
    //             _lock = op_lock.unwrap();
    //             break;
    //         }
    //         yield_();
    //     }
        
    //     let prio = self.priority;
    //     if prio == PRIO_NUM {
    //         self.currents[tid] = None;
    //         None
    //     } else {
    //         let cid = self.ready_queue[prio].pop_front().unwrap();
    //         let task = (*self.tasks.get(&cid).unwrap()).clone();
    //         if self.ready_queue[prio].is_empty() {
    //             self.bitmap.update(prio, false);
    //             self.priority = self.bitmap.get_priority();
    //         }
    //         self.currents[tid] = Some(cid);
    //         Some(task)
    //     }
    // }

    // 加入阻塞集合
    pub fn pending(&mut self, cid: usize) {
        let _lock = self.wr_lock.lock();
        self.pending_set.insert(cid);
    }

    // // 加入阻塞集合
    // pub fn pending_for_user(&mut self, cid: usize) {
    //     let mut op_lock;
    //     let mut _lock;
    //     while true {
    //         op_lock = self.wr_lock.try_lock();
    //         if op_lock.is_some() {
    //             _lock = op_lock.unwrap();
    //             break;
    //         }
    //         yield_();
    //     }
    //     self.pending_set.insert(cid);
    // }

    // 判断是否在阻塞集合中
    pub fn is_pending(&mut self, cid: usize) -> bool {
        let _lock = self.wr_lock.lock();
        self.pending_set.contains(&cid)
    }

    // // 判断是否在阻塞集合中
    // pub fn is_pending_for_user(&mut self, cid: usize) -> bool {
    //     let mut op_lock;
    //     let mut _lock;
    //     while true {
    //         op_lock = self.wr_lock.try_lock();
    //         if op_lock.is_some() {
    //             _lock = op_lock.unwrap();
    //             break;
    //         }
    //         yield_();
    //     }
    //     self.pending_set.contains(&cid)
    // }

    /// 增加执行器线程
    pub fn add_wait_tid(&mut self, tid: usize) {
        let _lock = self.wr_lock.lock();
        self.waits.push(tid);
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
        self.pending_set.remove(&cid.0);
        drop(lock);
        self.priority
    }

    // /// 阻塞协程重新入队
    // pub fn re_back_for_user(&mut self, cid: CoroutineId) -> usize {
    //     let mut op_lock;
    //     let mut _lock;
    //     while true {
    //         op_lock = self.wr_lock.try_lock();
    //         if op_lock.is_some() {
    //             _lock = op_lock.unwrap();
    //             break;
    //         }
    //         yield_();
    //     }
    //     let prio = self.tasks.get(&cid).unwrap().inner.lock().prio;
    //     self.ready_queue[prio].push_back(cid);
    //     self.bitmap.update(prio, true);
    //     if prio < self.priority {
    //         self.priority = prio;
    //     }
    //     self.pending_set.remove(&cid.0);
    //     self.priority
    // }

    /// 删除协程，协程已经被执行完了，在 fetch 取出 id 是就已经更新位图了，因此，这时不需要更新位图
    pub fn del_coroutine(&mut self, cid: CoroutineId) {
        let lock = self.wr_lock.lock();
        self.tasks.remove(&cid);
        drop(lock);
    }
}