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
mod syscall6;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use syscall::*;
pub use syscall6::*;
mod heap;
use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};


pub use trap::{UserTrapContext, UserTrapQueue, UserTrapRecord};

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(add_coroutine_addr: usize, is_waked_addr: usize, current_cid_addr: usize) {
// pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    use riscv::register::{mtvec::TrapMode, utvec};
    extern "C" {
        fn __alltraps_u();
    }
    unsafe {
        utvec::write(__alltraps_u as usize, TrapMode::Direct);
        ADD_COROUTINE_ADDR = add_coroutine_addr;
        IS_WAKED_ADDR = is_waked_addr;
        CURRENT_CID_ADDR = current_cid_addr;
    }
    heap::init();
    // main();
    add_coroutine(Box::pin(async{ main(); }), runtime::PRIO_NUM - 1);

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
    // exit(main());
}

static mut ADD_COROUTINE_ADDR: usize = 0;
static mut IS_WAKED_ADDR: usize = 0;
static mut CURRENT_CID_ADDR: usize = 0;

// 用户态添加协程
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    unsafe {
        let add_coroutine_fn: fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) = 
            core::mem::transmute(ADD_COROUTINE_ADDR);
        let pid = sys_getpid() as usize;
        add_coroutine_fn(future, prio, pid + 1);
    }
}

// 协程是否被唤醒过
pub fn is_waked(cid: usize) -> bool {
    unsafe {
        let is_waked_fn: fn(cid: usize) -> bool = 
            core::mem::transmute(IS_WAKED_ADDR);
        is_waked_fn(cid)
    }
}

// 当前正在运行的协程，只能在 协程内部使用，即在 async 块内使用
pub fn current_cid() -> usize {
    unsafe {
        let current_cid_fn: fn() -> usize = 
            core::mem::transmute(CURRENT_CID_ADDR);
        current_cid_fn()
    }
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

/******************** 异步系统调用 *********************************/
pub fn async_write(fd: usize, buffer_ptr: usize, buffer_len: usize, key: usize) -> isize {
    async_sys_write(fd, buffer_ptr, buffer_len, key)
}
pub fn async_read(fd: usize, buffer_ptr: usize, buffer_len: usize, key: usize, cid: usize) -> isize {
    async_sys_read(fd, buffer_ptr, buffer_len, key, cid)
}


// 异步系统调用
pub struct AsyncCall {
    call_type: usize,   // 系统调用类型，读 / 写
    fd: usize,          // 文件描述符
    buffer_ptr: usize,  // 缓冲区指针
    buffer_len: usize,  // 缓冲区长度
    key: usize,         // 类似于钥匙，读写的协程所拥有的钥匙必须要相同，这样才能够建立正确的对应关系，体现了协作
    cnt: usize,         
}

impl AsyncCall {
    pub fn new( call_type: usize, fd: usize, buffer_ptr: usize, buffer_len: usize, key: usize) -> Self {
        Self { 
            call_type, fd, buffer_ptr, buffer_len, key, cnt: 0
        }
    }
}

impl Future for AsyncCall {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // submit async task to kernel and return immediately
        if self.cnt == 0 {
            match self.call_type {
                ASYNC_SYSCALL_READ => async_sys_read(self.fd, self.buffer_ptr, self.buffer_len, self.key, current_cid()),
                ASYNC_SYSCALL_WRITE => async_sys_write(self.fd, self.buffer_ptr, self.buffer_len, self.key),
                _ => {0},
            };
            self.cnt += 1;
        }

        if is_waked(current_cid()) {
            println!("current coroutine is done {}", current_cid());
            return Poll::Ready(());
        } else {
            return Poll::Pending;
        }
    
    }
}