#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
// mod syscall;
pub mod trace;
pub mod trap;
pub mod user_uart;
mod syscall6;

extern crate alloc;

pub use syscall::*;
pub use syscall6::*;
mod heap;
use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};
use riscv::register::mtvec::TrapMode;
use riscv::register::{uie, utvec};


pub use trap::{UserTrapContext, UserTrapQueue, UserTrapRecord};

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() {
// pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    extern "C" {
        fn __alltraps_u();
    }
    unsafe {
        utvec::write(__alltraps_u as usize, TrapMode::Direct);
    }
    heap::init();
    unifi_exposure::init_unifi_sche(unsafe { INTERFACE_TABLE as usize });
    add_coroutine(Box::pin(async{ main(); }), unifi_exposure::PRIO_NUM - 1);
}

// 共享库的接口表地址，内核解析 elf 时填充
#[no_mangle]
#[link_section = ".bss.interface"]
static mut INTERFACE_TABLE: *mut usize = 0 as *mut usize;

// 用户态添加协程
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    let pid = getpid() as usize;
    unifi_exposure::add_coroutine(future, prio, pid + 1);
}

// 当前正在运行的协程，只能在协程内部使用，即在 async 块内使用
pub fn current_cid() -> usize {
    unifi_exposure::current_cid(false)
}

pub fn re_back(cid: usize) {
    let pid = getpid() as usize;
    unifi_exposure::re_back(cid, pid + 1);
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

pub fn init_user_trap() -> isize {
    let tid = thread_create(user_interrupt_handler as usize, 0);
    let ans = sys_init_user_trap(tid as usize);
    ans
}

fn user_interrupt_handler() {
    extern "C" {
        fn __alltraps_u();
    }
    unsafe {
        utvec::write(__alltraps_u as usize, TrapMode::Direct);
        uie::set_usoft();
        uie::set_utimer();
    }

    loop {
        hang();
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
            return Poll::Pending;
        }
        return Poll::Ready(());
        // if is_waked(current_cid()) {
        //     println!("current coroutine is done {}", current_cid());
        //     return Poll::Ready(());
        // } else {
        //     return Poll::Pending;
        // }
    
    }
}