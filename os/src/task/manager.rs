use super::TaskControlBlock;
use alloc::collections::{VecDeque, BTreeSet};
use alloc::sync::Arc;
use lib_so::max_prio_pid;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
    user_intr_process_set: BTreeSet<usize>
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            user_intr_process_set: BTreeSet::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }

    pub fn add_user_intr_task(&mut self, pid: usize) {
        self.user_intr_process_set.insert(pid);
    }

    #[allow(unused)]
    pub fn remove(&mut self, task: &Arc<TaskControlBlock>) {
        for (idx, task_item) in self.ready_queue.iter().enumerate() {
            if *task_item == *task {
                self.ready_queue.remove(idx);
                break;
            }
        }
    }

    pub fn remove_uintr_task(&mut self, pid: usize) {
        self.user_intr_process_set.remove(&pid);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // // May need to concern affinity
        // debug!("tasks total: {}", self.ready_queue.len());
        // // error!("max prio pid is {}", crate::lkm::max_prio_pid());
        if !self.user_intr_process_set.is_empty() {
            for pid in self.user_intr_process_set.iter() {
                lib_so::update_prio(pid + 1, 0);
                // info!("update prio: {}", pid);
            }
            // info!("fetch user intr task");
            // return (self.user_intr_task_queue.pop_back(), true);
        }
        let prio_pid = lib_so::max_prio_pid() - 1;
        // 如果内核协程的优先级最高，则
        // if prio_pid == 0 {
        //     return None;
        // }
        let n = self.ready_queue.len();
        if n == 0 { return None; }
        let mut peek;
        let mut cnt = 0;
        loop {
            peek = self.ready_queue.pop_front().unwrap();
            let pid = peek.process.upgrade().unwrap().getpid();
            if pid == prio_pid {
                return Some(peek);
            }
            self.ready_queue.push_back(peek);
            cnt += 1;
            if cnt >= n { break; }
        }
        self.ready_queue.pop_front()
    }

    #[allow(unused)]
    pub fn prioritize(&mut self, pid: usize) {
        let q = &mut self.ready_queue;
        if q.is_empty() || q.len() == 1 {
            return;
        }
        let front_pid = q.front().unwrap().process.upgrade().unwrap().pid.0;
        if front_pid == pid {
            debug!("[Taskmgr] Task {} already at front", pid);

            return;
        }
        q.rotate_left(1);
        while {
            let f_pid = q.front().unwrap().process.upgrade().unwrap().pid.0;
            f_pid != pid && f_pid != front_pid
        } {
            q.rotate_left(1);
        }
        if q.front().unwrap().process.upgrade().unwrap().pid.0 == pid {
            debug!("[Taskmgr] Prioritized task {}", pid);
        }
    }
}
