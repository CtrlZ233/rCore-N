#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(asm_sym)]

#[macro_use]
mod console;
mod syscall;

mod heap;
mod thread;
mod executor;
mod interface;

extern crate alloc;

use config::CPU_NUM;
use heap::MutAllocator;
use runtime::Executor;
use interface::{add_coroutine, poll_future};
use alloc::boxed::Box;
use alloc::vec;
use spin::Mutex;
use syscall::*;
use thread::Thread;
use crate::config::{ENTRY, UNFI_SCHE_BUFFER};

mod config;

// static mut ENTRY: [usize; CPU_NUM] = [0usize; CPU_NUM];

/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message().unwrap();
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}, {}",
            location.file(),
            location.line(),
            err
        );
    } else {
        println!("Panicked: {}", err);
    }
    sys_exit(-1);
}


/// _start() 函数由内核跳转执行，在每次执行线程之前需要由内核调用一次，设置默认的堆
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> usize {
    let heapptr = UNFI_SCHE_BUFFER;
    let heap = heapptr as *mut usize as *mut Mutex<MutAllocator<32>>;
    let exe = (heapptr + core::mem::size_of::<Mutex<MutAllocator<32>>>()) as *mut usize as *mut Executor;
    unsafe {
        heap::init(& *heap);
        executor::init(&mut *exe);
    }
    primary_thread as usize
}

/// sret 进入用户态的入口，在这个函数再执行 main 函数
fn primary_thread() {
    unsafe {
        let secondary_init: fn(usize) = core::mem::transmute(ENTRY);
        // main_addr 表示用户进程 main 函数的地址
        secondary_init(add_coroutine as usize);
    }
    // 主线程，在这里创建执行协程的线程，之后再进行控制
    // let mut thread = Thread::new();
    // thread.execute();
    let mut wait_tid = vec![];
    let max_len = 5;
    let pid = sys_getpid();
    if pid != 0 {
        for _ in 0..max_len {
            wait_tid.push(sys_thread_create(poll_future as usize, 0));
        }
    }

    poll_future(0);
    //
    for tid in wait_tid.iter() {
        waittid(*tid as usize);
    }
    sys_exit(0);
}

pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}



