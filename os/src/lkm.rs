use crate::loader::get_app_data_by_name;
use alloc::vec::Vec;
use crate::mm::{KERNEL_SPACE, MemorySet};
use lazy_static::*;
use alloc::sync::Arc;
use core::mem::transmute;
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;
use runtime::fun_offset::*;

lazy_static! {
    pub static ref UNFI_SCHE_DATA: Arc<Vec<u8>> = Arc::new(get_app_data_by_name("unfi-sche").unwrap().to_vec());
    pub static ref UNFI_SCHE_MEMORYSET: MemorySet = MemorySet::from_module(
        UNFI_SCHE_DATA.as_slice()
    );    
}

pub fn init(){
    // crate::println!("lkm init");
    debug!("lkm init");
    add_lkm_image();
    debug!("lkm init done");
    // crate::println!("lkm init done");
}

fn add_lkm_image(){

    KERNEL_SPACE.lock().add_kernel_module(&UNFI_SCHE_MEMORYSET);

    KERNEL_SPACE.lock().activate();
    debug!("unfi init");
    // 执行共享调度器的_start() 函数
    unsafe {
        let unfi_sche_start: fn() -> usize = transmute(UNFI_SCHE_START);
        INTERFACE_TABLE = unfi_sche_start() as *mut usize;
    }
    add_coroutine(Box::pin(async{ error!("add_coroutine"); }), 0);
    add_coroutine(Box::pin(async{ error!("add_coroutine"); }), 0);
    debug!("unfi init done");

}

pub const UNFI_SCHE_START: usize = 0x96000000usize;

// 共享调度器的模块接口表指针
#[no_mangle]
pub static mut INTERFACE_TABLE: *mut usize = 0 as *mut usize;

// 用户进程的调度器入口
pub fn user_entry() -> usize {
    unsafe {
        *INTERFACE_TABLE as usize
    }
}
// 获取优先级最高的用户进程
pub fn max_prio_pid() -> usize {
    unsafe {
        // error!("max_prio_pid ptr {:#x}", *interface.add(1) as usize);
        let max_prio_pid: fn() -> usize = transmute(*INTERFACE_TABLE.add(MAX_PRIO_PID) as usize);
        max_prio_pid()
    }
}

// 添加协程
pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize) {
    unsafe {
        let add_coroutine_fn: fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) = 
            transmute(*INTERFACE_TABLE.add(ADD_COROUTINE) as usize);
        add_coroutine_fn(future, prio, 0);
    }
}

// 运行内核协程
pub fn poll_kernel_future() {
    unsafe {
        let poll_kernel_future: fn() = transmute(*INTERFACE_TABLE.add(POLL_KERNEL_FUTURE) as usize);
        poll_kernel_future();
    }
}

// 唤醒内核协程
pub fn wake_kernel_future(cid: usize) {
    unsafe {
        let wake_kernel_future: fn(cid: usize, pid: usize) = transmute(*INTERFACE_TABLE.add(RE_BACK) as usize);
        wake_kernel_future(cid, 0);
    }
}

// 内核当前正在运行的协程
pub fn current_cid() -> usize {
    unsafe {
        let current_cid: fn() -> usize = transmute(*INTERFACE_TABLE.add(CURRENT_CID) as usize);
        current_cid()
    }
}




