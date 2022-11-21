//! 这个库暴露出共享调度器中使用的数据结构以及接口
//! 将 `Executor` 数据结构暴露出来，避免在内核和 user_lib 中重复定义
//! 进程需要在自己的地址空间中声明这个对象
//! 共享调度器通过 `Executor` 对象的虚拟地址来完成对应的操作
//! 
//! 暴露的接口会通过单例模式供内核和用户程序使用（内核和用户进程各自都有实例实例）
//! 这个模块需要手动指出接口的函数指针在 GOT 表中的偏移


#![no_std]

mod coroutine;
mod task_waker;
mod executor;
mod config;
mod bitmap;
mod fun_offset;

extern crate alloc;

pub use executor::Executor;
pub use coroutine::{CoroutineId, Coroutine};
pub use config::PRIO_NUM;
pub use config::CBQ_MAX;
pub use config::MAX_PROC_NUM;
use bitmap::BitMap;
// pub use task_waker::TaskWaker;
use spin::Once;
use core::future::Future;
use core::pin::Pin;
use alloc::boxed::Box;
use fun_offset::*;

// 共享调度器暴露的接口
pub struct UnifiScheFunc(usize);

impl UnifiScheFunc {
    fn user_entry(&self) -> usize {
        unsafe {
            *(self.0 as *mut usize)
        }
    }

    fn max_prio_pid(&self) -> usize {
        unsafe {
            let max_prio_pid: fn() -> usize = core::mem::transmute(*(self.0 as *mut usize).add(MAX_PRIO_PID));
            max_prio_pid()
        }
    }

    fn add_coroutine(&self, future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize){
        unsafe {
            let add_coroutine_true: fn(Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, usize, usize) = 
                core::mem::transmute(*(self.0 as *mut usize).add(ADD_COROUTINE));
            add_coroutine_true(future, prio, pid);
        }
    }

    pub fn poll_kernel_future(&self) {
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

    fn current_cid(&self) -> usize {
        unsafe {
            let current_cid_fn: fn() -> usize = 
            core::mem::transmute(*(self.0 as *mut usize).add(CURRENT_CID));
            current_cid_fn()
        }
    }
}

static UNIFI_SCHE: Once<UnifiScheFunc> = Once::new();

// 初始化
pub fn init_unifi_sche(interface_ptr: usize) {
    UNIFI_SCHE.call_once(|| UnifiScheFunc(interface_ptr));
}
// 用户进程的调度器入口
pub fn user_entry() -> usize {
    UNIFI_SCHE.get().unwrap().user_entry()
}
// 获取优先级最高的用户进程
pub fn max_prio_pid() -> usize {
    UNIFI_SCHE.get().unwrap().max_prio_pid()
}
// 添加协程
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize){
    UNIFI_SCHE.get().unwrap().add_coroutine(future, prio, pid);
}
// 运行内核协程
pub fn poll_kernel_future() {
    UNIFI_SCHE.get().unwrap().poll_kernel_future();
}
// 唤醒协程
pub fn re_back(cid: usize, pid: usize) {
    UNIFI_SCHE.get().unwrap().re_back(cid, pid);
}
// 获取当前正在运行的协程，只能在协程内部使用，即在 async 块内使用
pub fn current_cid() -> usize {
    UNIFI_SCHE.get().unwrap().current_cid()
}
