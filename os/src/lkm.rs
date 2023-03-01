use crate::loader::get_app_data_by_name;
use alloc::vec::Vec;
use crate::mm::{KERNEL_SPACE, MemorySet};
use lazy_static::*;
use alloc::sync::Arc;
use core::mem::transmute;
use alloc::boxed::Box;

lazy_static! {
    pub static ref UNFI_SCHE_DATA: Arc<Vec<u8>> = Arc::new(get_app_data_by_name("sharedscheduler").unwrap().to_vec());
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
        lib_so::init_sharedsche(INTERFACE_TABLE as usize);
        debug!("unfi init done {:#x}", INTERFACE_TABLE as usize);
    }

}

pub const UNFI_SCHE_START: usize = 0x96000000usize;

pub static mut INTERFACE_TABLE: *mut usize = 0 as *mut usize;












