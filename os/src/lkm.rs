use crate::{loader::get_app_data_by_name, task::pid2process, config::UNFI_SCHE_BUFFER};
use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
use unifi_exposure::{Executor, CoroutineId};
use crate::mm::{KERNEL_SPACE, MemorySet};
use lazy_static::*;
use alloc::sync::Arc;
use core::mem::transmute;
use alloc::boxed::Box;

use crate::{
    mm::translate_writable_va,
};

lazy_static! {
    pub static ref UNFI_SCHE_DATA: Arc<Vec<u8>> = Arc::new(get_app_data_by_name("unifi-sche").unwrap().to_vec());
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
    unifi_exposure::init_unifi_sche(unsafe { INTERFACE_TABLE as usize });
    unifi_exposure::add_coroutine(Box::pin(async{ error!("add_coroutine"); }), 0, 0);
    unifi_exposure::add_coroutine(Box::pin(async{ error!("add_coroutine"); }), 0, 0);
    debug!("unfi init done");

}

#[allow(unused)]
fn wake_cid(pid: usize, cid: usize) {
    let token = pid2process(pid).unwrap().acquire_inner_lock().get_user_token();
    let pa = translate_writable_va(token, UNFI_SCHE_BUFFER).unwrap();
    unsafe {
        let heapptr = *(pa as *const usize);
        let exe = (heapptr + core::mem::size_of::<LockedHeap>()) as *mut usize as *mut Executor;
        let prio = (*exe).re_back(CoroutineId(cid));
    }
    
}

pub const UNFI_SCHE_START: usize = 0x96000000usize;

// 共享调度器的模块接口表指针
#[no_mangle]
pub static mut INTERFACE_TABLE: *mut usize = 0 as *mut usize;











