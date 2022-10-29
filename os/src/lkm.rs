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

    // 执行共享调度器的_start() 函数
    unsafe {
        let unfi_sche_start: fn(usize, usize) -> usize = transmute(UNFI_SCHE_START);
        UNFI_SCHE_ENTRY = unfi_sche_start(0, 0);
        // crate::println!("primary init addr {:#x}", unfi_sche_start(0, 0));
    }

}


pub const UNFI_SCHE_START: usize = 0x96000000usize;

// 线程第一次进入用户态执行时的入口
pub static mut UNFI_SCHE_ENTRY: usize = 0;

pub fn task_init(entry: usize, heap_ptr: usize) {
    unsafe {
        let unfi_sche_start: fn(usize, usize) -> usize = transmute(UNFI_SCHE_START);
        UNFI_SCHE_ENTRY = unfi_sche_start(entry, heap_ptr);
    }
}
