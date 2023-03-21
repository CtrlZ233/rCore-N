#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
#[macro_use]
extern crate syscall;
mod lang_items;
pub mod trace;
pub mod trap;
pub mod user_uart;
pub mod matrix;

extern crate alloc;
pub use syscall::*;
mod heap;
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
    lib_so::spawn(move || async{ main(); }, lib_so::PRIO_NUM - 1, getpid() as usize + 1, lib_so::CoroutineKind::UserNorm);
}


// 当前正在运行的协程，只能在协程内部使用，即在 async 块内使用
pub fn current_cid() -> usize {
    lib_so::current_cid(false)
}

pub fn re_back(cid: usize) {
    let pid = getpid() as usize;
    lib_so::re_back(cid, pid + 1);
}

pub fn add_virtual_core() {
    lib_so::add_virtual_core();
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
