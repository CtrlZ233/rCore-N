use crate::loader::get_app_data_by_name;
use alloc::vec::Vec;
use crate::mm::{KERNEL_SPACE, MemorySet};
use lazy_static::*;
use alloc::sync::Arc;
use core::mem::transmute;
use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;

lazy_static! {
    pub static ref UNFI_SCHE_DATA: Arc<Vec<u8>> = Arc::new(get_app_data_by_name("unfi-sche").unwrap().to_vec());
    pub static ref UNFI_SCHE_MEMORYSET: MemorySet = MemorySet::from_module(
        UNFI_SCHE_DATA.as_slice()
    );    
}

pub fn init(){
    crate::println!("lkm init");
    add_lkm_image();
    crate::println!("lkm init done");
}

fn add_lkm_image(){

    KERNEL_SPACE.lock().add_kernel_module(&UNFI_SCHE_MEMORYSET);

    KERNEL_SPACE.lock().activate();
    // async fn test1() {
    //     log::debug!("43");
    // }
    // async fn test2() {
    //     log::debug!("444");
    // }
    // _start() 位于 0x87000000
    // 执行共享调度器的_start() 函数，填写好符号表
    // let basic_start_addr = 0x8700_0000usize;
    // unsafe {
    //     let basic_start: fn() = transmute(basic_start_addr);
    //     basic_start();
    // }
    // add_task_with_prority(Box::pin(test1()), 0, 0);
    // add_task_with_prority(Box::pin(test2()), 1, 0);
}


pub const SYMBOL_ADDR: *const usize = 0x87018000usize as *const usize;

pub fn add_task_with_prority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) {
    // log::warn!("kernel add task");
    unsafe {
        let add_task_with_prority: fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize, pid: usize) = 
            transmute(*SYMBOL_ADDR as usize);
        add_task_with_prority(future, prio, pid);
    }
} 

pub fn kernel_thread_main() {
    unsafe {
        let kernel_thread_main: fn() = transmute(*(SYMBOL_ADDR.add(1)) as usize);
        kernel_thread_main();
    }
}

pub fn update_global_bitmap() {
    unsafe {
        let update_global_bitmap: fn() = transmute(*(SYMBOL_ADDR.add(3)) as usize);
        update_global_bitmap();
    }
}

pub fn check_prio_pid(pid: usize) -> bool{
    unsafe {
        let check_prio_pid: fn(usize) -> bool = transmute(*(SYMBOL_ADDR.add(4)) as usize);
        check_prio_pid(pid)
    }
}

/// 根据 key 唤醒对应的 kernel_tid
pub fn wake_kernel_tid(pid: usize, key: usize) {
    // log::warn!("wake_kernel_tid");
    unsafe {
        let wake_kernel_tid: fn(pid: usize, key: usize) = transmute(*(SYMBOL_ADDR.add(5)) as usize);
        wake_kernel_tid(pid, key)
    }
}

/// 根据 write_tid 向 WRMAP 中注册 (write_tid, read_tid)
pub fn wrmap_register(key: usize, kernel_tid: usize) {
    // log::warn!("{} register kernel_tid {}", key, kernel_tid);
    unsafe {
        let wrmap_register: fn(key: usize, kernel_tid: usize) = transmute(*(SYMBOL_ADDR.add(6)) as usize);
        wrmap_register(key, kernel_tid)
    }
}

/// 获取内核目前正在运行的协程 id
pub fn kernel_current_corotine() -> usize {
    unsafe {
        let kernel_current_corotine: fn() -> usize = transmute(*(SYMBOL_ADDR.add(7)) as usize);
        kernel_current_corotine()
    }
}

pub fn add_callback(pid: usize, tid: usize) {
    unsafe {
        let add_callback: fn(usize, usize) = transmute(*(SYMBOL_ADDR.add(8)) as usize);
        add_callback(pid, tid)
    }
}


