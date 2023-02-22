//! 这个库暴露出共享调度器中使用的数据结构以及接口
//! 将 `Executor` 数据结构暴露出来，避免在内核和 user_lib 中重复定义
//! 进程需要在自己的地址空间中声明这个对象
//! 共享调度器通过 `Executor` 对象的虚拟地址来完成对应的操作
//! 
//! 暴露的接口会通过单例模式供内核和用户程序使用（内核和用户进程各自都有实例实例）
//! 这个模块需要手动指出接口的函数指针在 GOT 表中的偏移，因此在 `fun_offset` 中定义了一系列常量
//! `UnifiScheFunc(usize)` 表示共享调度器的接口实例


#![no_std]
#![deny(warnings, missing_docs)]

mod coroutine;
mod executor;
mod config;
mod bitmap;
mod fun_offset;

extern crate alloc;

pub use executor::Executor;
pub use coroutine::{CoroutineId, Coroutine};
pub use config::PRIO_NUM;
pub use config::MAX_PROC_NUM;
use bitmap::BitMap;
use spin::Once;
use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;
use fun_offset::*;

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
pub struct UnifiScheFunc(usize);

impl UnifiScheFunc {
    fn user_entry(&self) -> usize {
        unsafe {
            *(self.0 as *mut usize).add(USER_ENTRY)
        }
    }

    fn max_prio_pid(&self) -> usize {
        unsafe {
            let max_prio_pid: fn() -> usize = core::mem::transmute(*(self.0 as *mut usize).add(MAX_PRIO_PID));
            max_prio_pid()
        }
    }

    fn add_coroutine(&self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) -> usize {
        unsafe {
            let add_coroutine_true: fn(Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, usize, usize) -> usize = 
                core::mem::transmute(*(self.0 as *mut usize).add(ADD_COROUTINE));
            add_coroutine_true(future, prio, pid)
        }
    }

    fn poll_kernel_future(&self) {
        unsafe {
            let poll_kernel_future: fn() = core::mem::transmute(*(self.0 as *mut usize).add(POLL_KERNEL_FUTURE));
            poll_kernel_future();
        }
    }

    fn re_back(&self, cid: usize, pid: usize) {
        unsafe {
            let re_back_fn: fn(usize, usize) = core::mem::transmute(*(self.0 as *mut usize).add(RE_BACK));
            re_back_fn(cid, pid as usize);
        }
    }

    fn current_cid(&self, is_kernel: bool) -> usize {
        unsafe {
            let current_cid_fn: fn(bool) -> usize =
            core::mem::transmute(*(self.0 as *mut usize).add(CURRENT_CID));
            current_cid_fn(is_kernel)
        }
    }

    fn reprio(&self, cid: usize, prio: usize) {
        unsafe {
            let reprio_fn: fn(usize, usize) = core::mem::transmute(*(self.0 as *mut usize).add(REPRIO));
            reprio_fn(cid, prio);
        }
    }

    fn add_virtual_core(&self) {
        unsafe {
            let add_virtual_core_fn: fn() = core::mem::transmute(*(self.0 as *mut usize).add(ADD_VIRTUAL_CORE));
            add_virtual_core_fn();
        }
    }
}

static UNIFI_SCHE: Once<UnifiScheFunc> = Once::new();

/// 初始化
pub fn init_unifi_sche(interface_ptr: usize) {
    UNIFI_SCHE.call_once(|| UnifiScheFunc(interface_ptr));
}
/// 用户进程的调度器入口
pub fn user_entry() -> usize {
    UNIFI_SCHE.get().unwrap().user_entry()
}
/// 获取优先级最高的用户进程
pub fn max_prio_pid() -> usize {
    UNIFI_SCHE.get().unwrap().max_prio_pid()
}
/// 添加协程
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) -> usize {
    UNIFI_SCHE.get().unwrap().add_coroutine(future, prio, pid)
}
/// 运行内核协程
pub fn poll_kernel_future() {
    UNIFI_SCHE.get().unwrap().poll_kernel_future();
}
/// 唤醒协程
pub fn re_back(cid: usize, pid: usize) {
    UNIFI_SCHE.get().unwrap().re_back(cid, pid);
}
/// 获取当前正在运行的协程，只能在协程内部使用，即在 async 块内使用
pub fn current_cid(is_kernel: bool) -> usize {
    UNIFI_SCHE.get().unwrap().current_cid(is_kernel)
}
/// 更新协程优先级
pub fn reprio(cid: usize, prio: usize) {
    UNIFI_SCHE.get().unwrap().reprio(cid, prio);
}

/// 获取Poll入口地址
pub fn add_virtual_core() {
    UNIFI_SCHE.get().unwrap().add_virtual_core();
}