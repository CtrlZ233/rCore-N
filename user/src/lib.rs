#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
pub mod trace;
pub mod trap;
pub mod user_uart;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use alloc::vec::Vec;
use syscall::*;
mod heap;

pub use trap::{UserTrapContext, UserTrapQueue, UserTrapRecord};

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() {
// pub extern "C" fn _start(argc: usize, argv: usize) -> ! {

    use riscv::register::{mtvec::TrapMode, utvec};

    extern "C" {
        fn __alltraps_u();
    }
    unsafe {
        utvec::write(__alltraps_u as usize, TrapMode::Direct);
    }

    heap::init();

    // let mut v: Vec<&'static str> = Vec::new();
    // for i in 0..argc {
    //     let str_start =
    //         unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
    //     let len = (0usize..)
    //         .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
    //         .unwrap();
    //     v.push(
    //         core::str::from_utf8(unsafe {
    //             core::slice::from_raw_parts(str_start as *const u8, len)
    //         })
    //         .unwrap(),
    //     );
    // }
    exit(main());
    // main as usize
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
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
    sys_dup(fd)
}
pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits)
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
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

pub fn get_time() -> isize {
    let time = TimeVal::new();
    match sys_get_time(&time, 0) {
        0 => ((time.sec & 0xffff) * 1000 + time.usec / 1000) as isize,
        _ => -1,
    }
}

pub fn get_time_us() -> isize {
    let time = TimeVal::new();
    match sys_get_time(&time, 0) {
        0 => ((time.sec & 0xffff) * 1000_0000 + time.usec) as isize,
        _ => -1,
    }
}

pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str, args: &[*const u8]) -> isize {
    sys_exec(path, args)
}
pub fn spawn(path: &str) -> isize {
    sys_spawn(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
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
        match sys_waitpid(pid as isize, exit_code as *mut _) {
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
    sys_mailread(buf)
}

pub fn mailwrite(pid: usize, buf: &[u8]) -> isize {
    sys_mailwrite(pid, buf)
}

pub fn flush_trace() -> isize {
    sys_flush_trace()
}

pub fn init_user_trap() -> isize {
    sys_init_user_trap()
}

pub fn send_msg(pid: usize, msg: usize) -> isize {
    sys_send_msg(pid, msg)
}

pub fn set_timer(time_us: isize) -> isize {
    sys_set_timer(time_us)
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
    sys_gettid()
}
pub fn waittid(tid: usize) -> isize {
    loop {
        match sys_waittid(tid) {
            -2 => {
                yield_();
            }
            exit_code => return exit_code,
        }
    }
}
