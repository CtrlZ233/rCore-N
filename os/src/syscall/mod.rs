const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_MMAP: usize = 222;
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
const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
const SYSCALL_SEMAPHORE_UP: usize = 1021;
const SYSCALL_SEMAPHORE_DOWN: usize = 1022;
const SYSCALL_CONDVAR_CREATE: usize = 1030;
const SYSCALL_CONDVAR_SIGNAL: usize = 1031;
const SYSCALL_CONDVAR_WAIT: usize = 1032;

mod fs;
mod process;
mod thread;
mod async_wr;
mod sync;

use crate::trace::{push_trace, TRACE_SYSCALL_ENTER, TRACE_SYSCALL_EXIT};
use fs::*;
use process::*;
use sync::*;
pub use crate::syscall::thread::{sys_gettid, sys_thread_create, sys_waittid};
pub use async_wr::{WRMAP, AsyncKey};

pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    trace!("syscall {}, args {:x?}", syscall_id, args);
    push_trace(TRACE_SYSCALL_ENTER + syscall_id);
    let ret = match syscall_id {
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2], args[3], args[4]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0], args[1]),
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0] as isize),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_SPAWN => sys_spawn(args[0] as *const u8),
        SYSCALL_MAILREAD => sys_mailread(args[0] as *mut u8, args[1]),
        SYSCALL_MAILWRITE => sys_mailwrite(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_FLUSH_TRACE => sys_flush_trace(),
        SYSCALL_INIT_USER_TRAP => sys_init_user_trap(args[0]),
        SYSCALL_SEND_MSG => sys_send_msg(args[0], args[1]),
        SYSCALL_SET_TIMER => sys_set_timer(args[0]),
        SYSCALL_CLAIM_EXT_INT => sys_claim_ext_int(args[0]),
        SYSCALL_SET_EXT_INT_ENABLE => sys_set_ext_int_enable(args[0], args[1]),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_HANG => sys_hang(),
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_CONDVAR_CREATE => sys_condvar_create(args[0]),
        SYSCALL_CONDVAR_SIGNAL => sys_condvar_signal(args[0]),
        SYSCALL_CONDVAR_WAIT => sys_condvar_wait(args[0], args[1]),
        ASYNC_SYSCALL_WRITE => async_sys_write(args[0], args[1] as *const u8, args[2], args[3]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    };
    push_trace(TRACE_SYSCALL_EXIT + syscall_id);
    ret
}

/**************************** syscall6 ******************************************/
use async_wr::*;
use crate::syscall::thread::sys_hang;

pub const ASYNC_SYSCALL_WRITE: usize = 2502;
