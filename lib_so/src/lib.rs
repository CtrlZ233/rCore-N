#![no_std]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(atomic_from_mut, inline_const)]
#![feature(linkage)]
#![feature(alloc_error_handler)]
// #![deny(warnings, missing_docs)]


#[macro_use]
pub mod console;
#[macro_use]
pub mod kern_console;
pub mod config;
pub mod sharedsche;

extern crate alloc;
extern crate vdso_macro;

pub use vdso_macro::vdso;
pub use config::*;
pub use sharedsche::*;



/// Rust 异常处理函数，以异常方式关机。
#[cfg(feature = "inner")]
mod lang_items;
#[cfg(feature = "inner")]
pub use lang_items::*;



// 共享库的接口表地址，内核解析 elf 时填充
#[no_mangle]
#[link_section = ".bss.interface"]
static mut INTERFACE_TABLE: *mut usize = 0 as *mut usize;

use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;

/// 手动的指出各个函数地址在共享库接口中的偏移
pub const USER_ENTRY: usize     = 0;
pub const MAX_PRIO_PID: usize       = 1;
pub const ADD_COROUTINE: usize      = 2;
pub const POLL_KERNEL_FUTURE: usize = 3;
pub const RE_BACK: usize            = 4;
pub const CURRENT_CID: usize        = 5;
pub const REPRIO: usize             = 6;

/// 共享调度器暴露的接口
/// usize 表示暴露出的函数接口表的地址
/// 这里是模仿动态链接的过程，
/// 动态链接生成 GOT，GOT 表中保存动态链接的函数真正的地址，由内核解析 elf 时填充
/// 如果有多个用户程序使用了动态链接库，则每个用户程序编译出来的 elf 中都会有一份 GOT 表
/// Rust 目前是不支持 riscv 架构动态链接生成 GOT 的，因此在共享库中直接提供一个全局的函数接口表 INTERFACE_TABLE
/// 具体的每个函数在接口表中的偏移需要人工在 fun_offset 中指明
/// 用户程序或者内核需要使用共享库中提供的代码时，需要调用 init_unifi_sche 初始化这个接口实例，usize 表示链接进地址空间的接口表的虚拟地址
/// 初始化之后，直接使用这个模块暴露出来的接口，即可跳转到正确的位置执行共享库中的代码
/// 这个过程模拟了 PLT 中的函数找到 GOT 中正确的函数地址，跳转执行
/// 通过这种方式，不需要在内核和用户程序中都模拟这个过程，但是这种方式会带来调用上的开销，会增加一些指令
/// 初始化接口
pub fn init_sharedsche(interface_table_ptr: usize) {
    unsafe {
        INTERFACE_TABLE = interface_table_ptr as *mut usize;
    }
}
/// 用户进程的调度器入口
pub fn user_entry() -> usize {
    unsafe {
        *INTERFACE_TABLE.add(USER_ENTRY)
    }
}
/// 获取优先级最高的用户进程
pub fn max_prio_pid() -> usize {
    unsafe {
        let max_prio_pid: fn() -> usize = core::mem::transmute(*INTERFACE_TABLE.add(MAX_PRIO_PID));
        max_prio_pid()
    }
}

/// 运行内核协程
pub fn poll_kernel_future() {
    unsafe {
        let poll_kernel_future: fn() = core::mem::transmute(*INTERFACE_TABLE.add(POLL_KERNEL_FUTURE));
        poll_kernel_future();
    }
}
/// 唤醒协程
pub fn re_back(cid: usize, pid: usize) {
    unsafe {
        let re_back_fn: fn(usize, usize) = core::mem::transmute(*INTERFACE_TABLE.add(RE_BACK));
        re_back_fn(cid, pid as usize);
    }
}
/// 获取当前正在运行的协程，只能在协程内部使用，即在 async 块内使用
pub fn current_cid(is_kernel: bool) -> usize {
    unsafe {
        let current_cid_fn: fn(bool) -> usize =
        core::mem::transmute(*INTERFACE_TABLE.add(CURRENT_CID));
        current_cid_fn(is_kernel)
    }
}
/// 更新协程优先级
pub fn reprio(cid: usize, prio: usize) {
    unsafe {
        let reprio_fn: fn(usize, usize) = core::mem::transmute(*INTERFACE_TABLE.add(REPRIO));
        reprio_fn(cid, prio);
    }
}

/// 添加协程
pub fn spawn<F, T>(f: F, prio: usize, pid: usize, kind: CoroutineKind) 
where 
    F: FnOnce() -> T,
    T: Future<Output=()> + 'static + Send + Sync
{
    unsafe {
        let spawn_fn: fn(Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, usize, usize, CoroutineKind) = 
            core::mem::transmute(*INTERFACE_TABLE.add(ADD_COROUTINE));
            spawn_fn(Box::pin(f()), prio, pid, kind);
    }
}