//! 共享调度器模块

#![no_std]
#![no_main]
// #![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(atomic_from_mut, inline_const)]
#![deny(warnings, missing_docs)]

#[macro_use]
mod console;

mod heap;
mod executor;
mod prio_array;
mod config;

extern crate alloc;

use alloc::vec;
use executor::*;
use prio_array::max_prio_pid;
use syscall::*;
use crate::config::{ENTRY, MAX_THREAD_NUM};

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
    exit(-1);
}

// 自定义的模块接口，模块添加进地址空间之后，需要执行 _start() 函数填充这个接口表
static mut INTERFACE: [usize; 10] = [0; 10];

/// _start() 函数，返回接口表的地址
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() -> usize {
    unsafe {
        INTERFACE[0] = user_entry as usize;
        INTERFACE[1] = max_prio_pid as usize;
        INTERFACE[2] = add_coroutine as usize;
        INTERFACE[3] = poll_kernel_future as usize;
        INTERFACE[4] = re_back as usize;
        INTERFACE[5] = current_cid as usize;
        INTERFACE[6] = reprio as usize;
        &INTERFACE as *const [usize; 10] as usize
    }
}

/// sret 进入用户态的入口，在这个函数再执行 main 函数
fn user_entry() {
    unsafe {
        let secondary_init: fn(usize) = core::mem::transmute(ENTRY);
        // main_addr 表示用户进程 main 函数的地址
        secondary_init(&INTERFACE as *const [usize; 10] as usize);
    }
    // println!("test test");
    // 主线程，在这里创建执行协程的线程，之后再进行控制
    // let mut thread = Thread::new();
    // thread.execute();
    let mut wait_tid = vec![];
    // let max_len = MAX_THREAD_NUM - 2;
    // let max_len = 4;
    let max_len = 0;
    let pid = getpid();
    if pid == 0 {
        for _ in 0..max_len {
            wait_tid.push(thread_create(poll_user_future as usize, 0));
        }
    }
    let start = get_time();

    poll_user_future();
    for tid in wait_tid.iter() {
        waittid(*tid as usize);
    }
    let end = get_time();
    println!("total time: {} ms", end - start);
    
    exit(0);
}



