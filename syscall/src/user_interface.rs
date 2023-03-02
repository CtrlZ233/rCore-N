
use bitflags::bitflags;
use crate::*;

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
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

pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path.as_ptr() as usize, flags.bits as usize)
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn pipe(pipe: &mut [usize]) -> isize {
    sys_pipe(pipe.as_mut_ptr() as usize)
}

pub fn read(fd: usize, buffer: &mut [u8], key: usize, cid: usize) -> isize {
    sys_read(fd, buffer.as_mut_ptr() as usize, buffer.len(), key, cid)
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer.as_ptr() as usize, buffer.len())
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code as usize);
    panic!("sys_exit never returns!");
}

pub fn yield_() -> isize {
    sys_yield()
}

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

#[allow(unused_variables)]
pub fn get_time() -> isize {
    let time = TimeVal::new();
    match sys_get_time(&time as *const _ as usize, 0) {
        0 => ((time.sec & 0xffff) * 1000 + time.usec / 1000) as isize,
        _ => -1,
    }
}

pub fn get_time_us() -> isize {
    let time = TimeVal::new();
    match sys_get_time(&time as *const _ as usize, 0) {
        0 => ((time.sec & 0xffff) * 1000_0000 + time.usec) as isize,
        _ => -1,
    }
}

pub fn getpid() -> isize {
    sys_get_pid()
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str, args: &[*const u8]) -> isize {
    sys_exec(path.as_ptr() as usize, args.as_ptr() as usize)
}

pub fn spawn(path: &str) -> isize {
    sys_spawn(path.as_ptr() as usize)
}

pub fn wait(exit_code: *mut i32) -> isize {
    loop {
        match sys_wait_pid(usize::MAX, exit_code as usize) {
            -2 => {
                sys_yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_wait_pid(pid, exit_code as *mut _ as usize) {
            -2 => {
                sys_yield();
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
    sys_mail_read(buf.as_mut_ptr() as usize, buf.len())
}

pub fn mailwrite(pid: usize, buf: &[u8]) -> isize {
    sys_mail_write(pid, buf.as_ptr() as usize, buf.len())
}

pub fn flush_trace() -> isize {
    sys_flush_trace()
}

pub fn init_user_trap(tid: usize) -> isize {
    sys_init_user_trap(tid)
}

pub fn send_msg(pid: usize, msg: usize) -> isize {
    sys_send_msg(pid, msg)
}

pub fn set_timer(time_us: isize) -> isize {
    sys_set_timer(time_us as usize)
}

pub fn claim_ext_int(device_id: usize) -> isize {
    sys_claim_ext_int(device_id)
}

pub fn set_ext_int_enable(device_id: usize, enable: usize) -> isize {
    sys_set_ext_int_enable(device_id, enable)
}

pub fn thread_create(entry: usize, arg: usize) -> isize {
    sys_thread_create(entry, arg)
}

pub fn gettid() -> isize {
    sys_get_tid()
}

pub fn waittid(tid: usize) -> isize {
    loop {
        match sys_wait_tid(tid) {
            -2 => {
                sys_yield();
            }
            exit_code => return exit_code,
        }
    }
    
}

pub fn hang() {
    sys_hang();
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


pub fn mutex_create() -> isize {
    sys_mutex_create(false as usize)
}

pub fn mutex_blocking_create() -> isize {
    sys_mutex_create(true as usize)
}

pub fn mutex_lock(mutex_id: usize) {
    sys_mutex_lock(mutex_id);
}

pub fn mutex_unlock(mutex_id: usize) {
    sys_mutex_unlock(mutex_id);
}

pub fn condvar_create() -> isize {
    sys_condvar_create(0)
}

pub fn condvar_signal(condvar_id: usize) {
    sys_condvar_signal(condvar_id);
}

pub fn condvar_wait(condvar_id: usize, mutex_id: usize) {
    sys_condvar_wait(condvar_id, mutex_id);
}


pub fn async_read(fd: usize, buffer_ptr: usize, buffer_len: usize, key: usize, cid: usize) -> isize {
    sys_async_read(fd, buffer_ptr, buffer_len, key, cid)
}

pub fn async_write(fd: usize, buffer_ptr: usize, buffer_len: usize, key: usize) -> isize {
    sys_async_write(fd, buffer_ptr, buffer_len, key)
}