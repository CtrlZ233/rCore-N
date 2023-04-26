#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(map_try_insert)]
#![feature(vec_into_raw_parts)]
#![allow(unused)]


extern crate alloc;
extern crate rv_plic;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

use crate::{config::CPU_NUM, mm::init_kernel_space, sbi::send_ipi};
use core::arch::{asm, global_asm};

#[macro_use]
mod console;
mod config;
#[macro_use]
mod fs;
mod lang_items;
mod loader;
mod logger;
mod mm;
mod sbi;
mod syscall;
mod task;
mod sync;
mod timer;
mod trap;
#[macro_use]
mod trace;
mod lkm;
mod device;
mod net;

use alloc::vec;

use device::{plic, uart};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
pub fn rust_main(hart_id: usize, device_tree_addr: usize) -> ! {
    if hart_id == 0 {
        
        clear_bss();
        logger::init();
        mm::init();
        debug!("[kernel {}] Hello, world!", hart_id);
        debug!("device_tree_addr: {:#x}", device_tree_addr);
        mm::remap_test();
        trace::init();
        trace::trace_test();
        trap::init();
        // device::init_dt(device_tree_addr);
        device::init();
        plic::init();
        plic::init_hart(hart_id);
        uart::init();
        lkm::init();
        debug!("test end");
        extern "C" {
            fn boot_stack();
            fn boot_stack_top();
        }

        debug!(
            "boot_stack {:#x} top {:#x}",
            boot_stack as usize, boot_stack_top as usize
        );

        debug!("trying to add initproc");
        task::add_initproc();
        debug!("initproc added to task manager!");

        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            asm!("mv {}, sp", out(reg) sp);
            println_hart!("satp: {:#x}, sp: {:#x}", hart_id, satp, sp);
        }

        for i in 1..CPU_NUM {
            debug!("[kernel {}] Start {}", hart_id, i);
            let mask: usize = 1 << i;
            send_ipi(&mask as *const _ as usize);
        }
    } else {
        let hart_id = task::hart_id();

        init_kernel_space();

        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            asm!("mv {}, sp", out(reg) sp);
            println_hart!("satp: {:#x}, sp: {:#x}", hart_id, satp, sp);
        }
        trap::init();
        plic::init_hart(hart_id);
    }

    println_hart!("Hello", hart_id);

    timer::set_next_trigger();

    if hart_id == 0 {
        loader::list_apps();
    }
    lib_so::spawn(move || task::run_tasks(), 7, 0, lib_so::CoroutineKind::KernSche);
    lib_so::poll_kernel_future();
    panic!("Unreachable in rust_main!");
}
