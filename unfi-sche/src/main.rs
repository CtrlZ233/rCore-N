#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions, asm_sym)]
#![feature(panic_info_message)]


#[macro_use]
mod console;
mod syscall;

mod heap;
mod thread;
mod executor;
mod interface;

extern crate alloc;

use heap::MutAllocator;
use runtime::Executor;
use interface::{add_coroutine, run};
use alloc::boxed::Box;
use syscall::*;

static mut ENTRY: usize = 0usize;

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
unsafe extern "C" fn _start(entry: usize, heapptr: usize) -> usize {
    let heap = heapptr as *mut usize as *mut MutAllocator<32>;
    let exe = (heapptr + core::mem::size_of::<MutAllocator<32>>()) as *mut usize as *mut Executor;
    unsafe {
        heap::init(&mut *heap);
        executor::init(&mut *exe);
        ENTRY = entry;
    }
    primary_thread as usize
}

/// sret 进入用户态的入口，在这个函数再执行 main 函数
fn primary_thread() {
    println!("main thread init ");
    unsafe {
        println!("SECONDARY_ENTER {:#x}", ENTRY);
        let secondary_init: fn() = core::mem::transmute(ENTRY);
        secondary_init();
        // let main_entry =  secondary_init();
        // add_coroutine(Box::pin(test(main_entry)), 0);
    }
    // run();
    // sys_exit(0);
}

// async fn test(entry: usize) {
//     unsafe {
//         let secondary_thread: fn() -> usize = core::mem::transmute(entry);
//         secondary_thread();
//     }
// }



