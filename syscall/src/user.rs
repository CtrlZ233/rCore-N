use crate::syscall_asm::*;

const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_SPAWN: usize = 400;
const SYSCALL_MAILREAD: usize = 401;
const SYSCALL_MAILWRITE: usize = 402;
const SYSCALL_FLUSH_TRACE: usize = 555;
const SYSCALL_INIT_USER_TRAP: usize = 600;
const SYSCALL_SEND_MSG: usize = 601;
const SYSCALL_SET_TIMER: usize = 602;
const SYSCALL_CLAIM_EXT_INT: usize = 603;
const SYSCALL_SET_EXT_INT_ENABLE: usize = 604;

const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;
const SYSCALL_HANG: usize = 1003;

const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;

const SYSCALL_CONDVAR_CREATE: usize = 1030;
const SYSCALL_CONDVAR_SIGNAL: usize = 1031;
const SYSCALL_CONDVAR_WAIT: usize = 1032;
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        TimeVal { sec: 0, usec: 0 }
    }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}



pub fn dup(fd: usize) -> isize {
    unsafe { syscall1(SYSCALL_DUP, fd) }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    unsafe { syscall2(SYSCALL_OPEN, path.as_ptr() as usize, flags.bits as usize) }
}

pub fn close(fd: usize) -> isize {
    unsafe { syscall1(SYSCALL_CLOSE, fd) }
}

pub fn pipe(pipe: &mut [usize]) -> isize {
    unsafe { syscall1(SYSCALL_PIPE, pipe.as_mut_ptr() as usize) }
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    unsafe { syscall3(SYSCALL_READ, fd, buffer.as_mut_ptr() as usize, buffer.len()) }
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    unsafe { syscall3(SYSCALL_WRITE, fd, buffer.as_ptr() as usize, buffer.len()) }
}

pub fn exit(exit_code: i32) -> ! {
    unsafe { syscall1(SYSCALL_EXIT, exit_code as usize); }
    panic!("sys_exit never returns!");
}

pub fn yield_() -> isize {
    unsafe { syscall0(SYSCALL_YIELD) }
}

#[allow(unused_variables)]
pub fn get_time() -> isize {
    let time = TimeVal::new();
    match unsafe { syscall2(SYSCALL_GET_TIME, &time as *const _ as usize, 0) } {
        0 => ((time.sec & 0xffff) * 1000 + time.usec / 1000) as isize,
        _ => -1,
    }
}

pub fn get_time_us() -> isize {
    let time = TimeVal::new();
    match unsafe { syscall2(SYSCALL_GET_TIME, &time as *const _ as usize, 0) } {
        0 => ((time.sec & 0xffff) * 1000_0000 + time.usec) as isize,
        _ => -1,
    }
}

pub fn getpid() -> isize {
    unsafe { syscall0(SYSCALL_GETPID) }
}

pub fn fork() -> isize {
    unsafe { syscall0(SYSCALL_FORK) }
}

pub fn exec(path: &str, args: &[*const u8]) -> isize {
    unsafe { syscall2(SYSCALL_EXEC, path.as_ptr() as usize, args.as_ptr() as usize) }
}

pub fn spawn(path: &str) -> isize {
    unsafe { syscall1(SYSCALL_SPAWN, path.as_ptr() as usize) }
}

pub fn wait(exit_code: *mut i32) -> isize {
    loop {
        match unsafe { syscall2(SYSCALL_WAITPID, usize::MAX, exit_code as usize) } {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match unsafe { syscall2(SYSCALL_WAITPID, pid, exit_code as *mut _ as usize) } {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn sleep(period_ms: usize) {
    let start = get_time();
    while get_time() < start + period_ms as isize {
        // sys_yield();
    }
}

pub fn mailread(buf: &mut [u8]) -> isize {
    unsafe { syscall2(SYSCALL_MAILREAD, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn mailwrite(pid: usize, buf: &[u8]) -> isize {
    unsafe { syscall3(SYSCALL_MAILWRITE, pid, buf.as_ptr() as usize, buf.len()) }
}

pub fn flush_trace() -> isize {
    unsafe { syscall0(SYSCALL_FLUSH_TRACE) }
}

pub fn sys_init_user_trap(tid: usize) -> isize {
    unsafe { syscall1(SYSCALL_INIT_USER_TRAP, tid) }
}

pub fn send_msg(pid: usize, msg: usize) -> isize {
    unsafe { syscall2(SYSCALL_SEND_MSG, pid, msg) }
}

pub fn set_timer(time_us: isize) -> isize {
    unsafe { syscall1(SYSCALL_SET_TIMER, time_us as usize) }
}

pub fn claim_ext_int(device_id: usize) -> isize {
    unsafe { syscall1(SYSCALL_CLAIM_EXT_INT, device_id) }
}

pub fn set_ext_int_enable(device_id: usize, enable: usize) -> isize {
    unsafe { syscall2(SYSCALL_SET_EXT_INT_ENABLE, device_id, enable) }
}

pub fn thread_create(entry: usize, arg: usize) -> isize {
    unsafe { syscall2(SYSCALL_THREAD_CREATE, entry, arg) }
}

pub fn gettid() -> isize {
    unsafe { syscall0(SYSCALL_GETTID) }
}

pub fn waittid(tid: usize) -> isize {
    loop {
        match unsafe { syscall1(SYSCALL_WAITTID, tid) } {
            -2 => {
                yield_();
            }
            exit_code => return exit_code,
        }
    }
    
}

pub fn hang() {
    unsafe { syscall0(SYSCALL_HANG); }
}

pub fn sys_mutex_create(blocking: bool) -> isize {
    unsafe {
        syscall1(SYSCALL_MUTEX_CREATE, blocking as usize)
    }
}

pub fn sys_mutex_lock(id: usize) -> isize {
    unsafe {
        syscall1(SYSCALL_MUTEX_LOCK, id)
    }
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    unsafe {
        syscall1(SYSCALL_MUTEX_UNLOCK, id)
    }
}

// pub fn sys_semaphore_create(res_count: usize) -> isize {
//     unsafe {
//         syscall1(SYSCALL_SEMAPHORE_CREATE, res_count)
//     }
    
// }

// pub fn sys_semaphore_up(sem_id: usize) -> isize {
//     unsafe {
//         syscall1(SYSCALL_SEMAPHORE_UP, sem_id)
//     }
// }

// pub fn sys_semaphore_down(sem_id: usize) -> isize {
//     unsafe {
//         syscall1(SYSCALL_SEMAPHORE_DOWN, sem_id)
//     }
// }

pub fn sys_condvar_create(_arg: usize) -> isize {
    unsafe {
        syscall1(SYSCALL_CONDVAR_CREATE, _arg)
    }   
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    unsafe {
        syscall1(SYSCALL_CONDVAR_SIGNAL, condvar_id)
    }
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    unsafe {
        syscall2(SYSCALL_CONDVAR_WAIT, condvar_id, mutex_id)
    }
}

