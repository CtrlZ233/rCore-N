use alloc::string::String;
use spin::{Mutex, MutexGuard};
use crate::mm::{KERNEL_SPACE, MemorySet, PhysAddr, PhysPageNum, translate_writable_va, translated_refmut, VirtAddr};
use crate::task::{add_task, pid_alloc, PidHandle, TaskControlBlock};
use super::pid::RecycleAllocator;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use crate::config::{PAGE_SIZE, USER_TRAP_BUFFER};
use crate::fs::{File, Stdin, Stdout};
use crate::syscall::sys_gettid;
use crate::task::pool::insert_into_pid2process;
use crate::trap::{trap_handler, TrapContext, UserTrapInfo, UserTrapQueue};

pub struct ProcessControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: Mutex<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub is_zombie: bool,
    pub is_sstatus_uie: bool,
    pub memory_set: MemorySet,
    pub user_trap_info: Option<UserTrapInfo>,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub tasks: Vec<Option<Arc<TaskControlBlock>>>,
    pub task_res_allocator: RecycleAllocator,
}

impl ProcessControlBlockInner {
    #[allow(unused)]
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }

    pub fn is_zombie(&self) -> bool {
        self.is_zombie
    }

    pub fn alloc_tid(&mut self) -> usize {
        self.task_res_allocator.alloc()
    }

    pub fn dealloc_tid(&mut self, tid: usize) {
        self.task_res_allocator.dealloc(tid)
    }

    pub fn thread_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn get_task(&self, tid: usize) -> Arc<TaskControlBlock> {
        self.tasks[tid].as_ref().unwrap().clone()
    }

    pub fn mmap(&mut self, start: usize, len: usize, port: usize) -> Result<isize, isize> {
        self.memory_set.mmap(start, len, port)
    }

    pub fn munmap(&mut self, start: usize, len: usize) -> Result<isize, isize> {
        self.memory_set.munmap(start, len)
    }

    pub fn is_user_trap_enabled(&self) -> bool {
        self.is_sstatus_uie
    }

    pub fn init_user_trap(&mut self) -> Result<isize, isize> {
        use riscv::register::sstatus;
        if self.user_trap_info.is_none() {
            // R | W
            if self.mmap(USER_TRAP_BUFFER, PAGE_SIZE, 0b11).is_ok() {
                let phys_addr =
                    translate_writable_va(self.get_user_token(), USER_TRAP_BUFFER).unwrap();
                self.user_trap_info = Some(UserTrapInfo {
                    user_trap_buffer_ppn: PhysPageNum::from(PhysAddr::from(phys_addr)),
                    devices: Vec::new(),
                });
                let trap_queue = self.user_trap_info.as_mut().unwrap().get_trap_queue_mut();
                *trap_queue = UserTrapQueue::new();
                unsafe {
                    sstatus::set_uie();
                }
                self.is_sstatus_uie = true;
                return Ok(USER_TRAP_BUFFER as isize);
            } else {
                warn!("[init user trap] mmap failed!");
            }
        } else {
            warn!("[init user trap] self user trap info is not None!");
        }
        Err(-1)
    }

    pub fn restore_user_trap_info(&mut self) {
        use riscv::register::{uip, uscratch};
        if self.is_user_trap_enabled() && sys_gettid() == 0 {
            if let Some(trap_info) = &mut self.user_trap_info {
                // if trap_info.user_trap_record_num > 0 {
                //     uscratch::write(trap_info.user_trap_record_num as usize);
                //     trap_info.user_trap_record_num = 0;
                //     unsafe {
                //         uip::set_usoft();
                //     }
                // }
                if !trap_info.get_trap_queue().is_empty() {
                    trace!("restore {} user trap", trap_info.user_trap_record_num());
                    uscratch::write(trap_info.user_trap_record_num());
                    unsafe {
                        uip::set_usoft();
                    }
                }
            }
        }
    }
}

impl ProcessControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<ProcessControlBlockInner> {
        self.inner.lock()
    }

    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        debug!("new space end");
        // allocate a pid
        let pid_handle = pid_alloc();
        let process = Arc::new(Self {
            pid: pid_handle,
            inner: unsafe {
                Mutex::new(ProcessControlBlockInner {
                    is_zombie: false,
                    is_sstatus_uie: false,
                    memory_set,
                    user_trap_info: None,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                })
            },
        });
        debug!("start alloc user resource");
        // create a main thread, we should allocate ustack and trap_cx here
        let task = Arc::new(TaskControlBlock::new(
            Arc::clone(&process),
            ustack_base,
            true,
        ));
        debug!("alloc user resource end");
        // prepare trap_cx of main thread
        let task_inner = task.acquire_inner_lock();
        let trap_cx = task_inner.get_trap_cx();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = task.kstack.get_top();
        drop(task_inner);
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            ustack_top,
            KERNEL_SPACE.lock().token(),
            kstack_top,
            trap_handler as usize,
        );
        // add main thread to the process
        let mut process_inner = process.acquire_inner_lock();
        process_inner.tasks.push(Some(Arc::clone(&task)));
        drop(process_inner);
        insert_into_pid2process(process.getpid(), Arc::clone(&process));
        // add main thread to scheduler
        add_task(task);
        process
    }

    /// Only support processes with a single thread.
    pub fn exec(self: &Arc<Self>, elf_data: &[u8]) {
        assert_eq!(self.acquire_inner_lock().thread_count(), 1);
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        let new_token = memory_set.token();
        // substitute memory_set
        self.acquire_inner_lock().memory_set = memory_set;
        self.acquire_inner_lock().user_trap_info = None;
        // then we alloc user resource for main thread again
        // since memory_set has been changed
        let task = self.acquire_inner_lock().get_task(0);
        let mut task_inner = task.acquire_inner_lock();
        task_inner.res.as_mut().unwrap().ustack_base = ustack_base;
        task_inner.res.as_mut().unwrap().alloc_user_res();
        task_inner.trap_cx_ppn = task_inner.res.as_mut().unwrap().trap_cx_ppn();
        let mut user_sp = task_inner.res.as_mut().unwrap().ustack_top();
        // initialize trap_cx
        let mut trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            task.kstack.get_top(),
            trap_handler as usize,
        );
        // trap_cx.x[10] = args.len();
        // trap_cx.x[11] = argv_base;
        *task_inner.get_trap_cx() = trap_cx;
    }

    /// Only support processes with a single thread.
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        debug!("fork start inner");
        let mut parent = self.acquire_inner_lock();
        assert_eq!(parent.thread_count(), 1);
        // clone parent's memory_set completely including trampoline/ustacks/trap_cxs
        let memory_set = MemorySet::from_existed_user(&parent.memory_set);
        // alloc a pid
        let pid = pid_alloc();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        debug!("fork start inner2");

        let mut user_trap_info: Option<UserTrapInfo> = None;
        if let Some(mut trap_info) = parent.user_trap_info.clone() {
            debug!("[fork] copy parent trap info");
            trap_info.user_trap_buffer_ppn = memory_set
                .translate(VirtAddr::from(USER_TRAP_BUFFER).into())
                .unwrap()
                .ppn();
            user_trap_info = Some(trap_info);
        }

        debug!("fork start inner3");

        // create child process pcb
        let child = Arc::new(Self {
            pid,
            inner: unsafe {
                Mutex::new(ProcessControlBlockInner {
                    is_zombie: false,
                    is_sstatus_uie: false,
                    memory_set,
                    user_trap_info: None,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                })
            },
        });
        debug!("fork start inner4");
        // add child
        parent.children.push(Arc::clone(&child));
        // create main thread of child process
        let task = Arc::new(TaskControlBlock::new(
            Arc::clone(&child),
            parent
                .get_task(0)
                .acquire_inner_lock()
                .res
                .as_ref()
                .unwrap()
                .ustack_base(),
            // here we do not allocate trap_cx or ustack again
            // but mention that we allocate a new kstack here
            false,
        ));
        debug!("fork start inner5");
        drop(parent);
        // attach task to child process
        let mut child_inner = child.acquire_inner_lock();
        child_inner.tasks.push(Some(Arc::clone(&task)));
        drop(child_inner);
        // modify kstack_top in trap_cx of this thread
        let task_inner = task.acquire_inner_lock();
        let trap_cx = task_inner.get_trap_cx();
        trap_cx.kernel_sp = task.kstack.get_top();
        debug!("fork start inner6");
        drop(task_inner);
        insert_into_pid2process(child.getpid(), Arc::clone(&child));
        debug!("fork start inner7");
        child
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}